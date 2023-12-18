#[macro_use]
extern crate serde;
use candid::{Decode, Encode};
use ic_cdk::api::time;
use ic_cdk::caller;
use ic_stable_structures::memory_manager::{MemoryId, MemoryManager, VirtualMemory};
use ic_stable_structures::{BoundedStorable, Cell, DefaultMemoryImpl, StableBTreeMap, Storable};
use std::{borrow::Cow, cell::RefCell};

type Memory = VirtualMemory<DefaultMemoryImpl>;
type IdCell = Cell<u64, Memory>;

#[derive(candid::CandidType, Clone, Serialize, Deserialize, Default)]
struct LegalConsultation {
    id: u64,
    client_details: ClientDetails,
    advisor_id: u64,
    details: String,
    created_at: u64,
    closed_at: Option<u64>,
    is_completed: bool,
}

#[derive(candid::CandidType, Clone, Serialize, Deserialize, Default)]
struct LegalConsultationPayload {
    client_details: ClientDetails,
    advisor_id: u64,
    details: String,
}

#[derive(candid::CandidType, Clone, Serialize, Deserialize, Default)]
struct ClientDetails {
    name: String,
    email: String
}

impl Storable for LegalConsultation {
    fn to_bytes(&self) -> Cow<[u8]> {
        Cow::Owned(Encode!(self).unwrap())
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
    }
}

impl BoundedStorable for LegalConsultation {
    const MAX_SIZE: u32 = 1024;  // Set an appropriate maximum size
    const IS_FIXED_SIZE: bool = false;  // Set to true if the size is fixed, otherwise false
}

#[derive(candid::CandidType, Clone, Serialize, Deserialize, Default)]
struct LegalAdvisor {
    id: u64,
    name: String,
    principal_string: String,
    credentials: String,
    rating: u32,
}
#[derive(candid::CandidType, Clone, Serialize, Deserialize, Default)]
struct LegalAdvisorPayload {
    name: String,
    credentials: String,
    rating: u32,
}

impl Storable for LegalAdvisor {
    fn to_bytes(&self) -> Cow<[u8]> {
        Cow::Owned(Encode!(self).unwrap())
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
    }
}

impl BoundedStorable for LegalAdvisor {
    const MAX_SIZE: u32 = 1024;  // Set an appropriate maximum size
    const IS_FIXED_SIZE: bool = false;  // Set to true if the size is fixed, otherwise false
}

thread_local! {
    static MEMORY_MANAGER: RefCell<MemoryManager<DefaultMemoryImpl>> = RefCell::new(
        MemoryManager::init(DefaultMemoryImpl::default())
    );

    static ID_COUNTER: RefCell<IdCell> = RefCell::new(
        IdCell::init(MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(0))), 0)
            .expect("Cannot create a counter")
    );

    static LEGAL_CONSULTATIONS: RefCell<StableBTreeMap<u64, LegalConsultation, Memory>> =
        RefCell::new(StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(1)))
    ));

    static LEGAL_ADVISORS: RefCell<StableBTreeMap<u64, LegalAdvisor, Memory>> =
        RefCell::new(StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(2)))
    ));
}


#[ic_cdk::query]
fn get_legal_consultation(id: u64) -> Result<LegalConsultation, Error> {
    match _get_legal_consultation(&id) {
        Some(consultation) => Ok(consultation),
        None => Err(Error::NotFound {
            msg: format!("Legal consultation with id={} not found", id),
        }),
    }
}

#[ic_cdk::update]
fn initiate_legal_consultation(payload: LegalConsultationPayload) -> Result<LegalConsultation, Error> {
    validate_legal_consultation_payload(&payload)?;

    let advisor = get_legal_advisor(payload.advisor_id)?;
    is_caller_advisor(&advisor)?;
    let id = ID_COUNTER
        .with(|counter| {
            let current_value = *counter.borrow().get();
            counter.borrow_mut().set(current_value + 1)
        })
        .expect("Cannot increment id counter");

    let consultation = LegalConsultation {
        id,
        client_details: payload.client_details,
        advisor_id: payload.advisor_id,
        details: payload.details,
        created_at: time(),
        closed_at: None,
        is_completed: false,
    };

    do_insert_legal_consultation(&consultation);
    Ok(consultation)
}

#[ic_cdk::update]
fn update_legal_advisor(id: u64, payload: LegalAdvisorPayload) -> Result<LegalAdvisor, Error> {
    let mut legal_advisor = get_legal_advisor(id)?;
    is_caller_advisor(&legal_advisor)?;

    validate_legal_advisor_payload(&payload)?;

    legal_advisor.name = payload.name;
    legal_advisor.credentials = payload.credentials;
    legal_advisor.rating = payload.rating;

    do_insert_legal_advisor(&legal_advisor);
    Ok(legal_advisor)
}

fn do_insert_legal_consultation(consultation: &LegalConsultation) {
    LEGAL_CONSULTATIONS.with(|service| service.borrow_mut().insert(consultation.id, consultation.clone()));
}

fn _get_legal_consultation(id: &u64) -> Option<LegalConsultation> {
    LEGAL_CONSULTATIONS.with(|service| service.borrow().get(id))
}

#[ic_cdk::update]
fn delete_legal_consultation(id: u64) -> Result<(), Error> {
    if let Some(consultation) = _get_legal_consultation(&id) {
        let advisor = get_legal_advisor(consultation.advisor_id)?;
        is_caller_advisor(&advisor)?;
        LEGAL_CONSULTATIONS.with(|service| service.borrow_mut().remove(&id));
        Ok(())
    } else {
        Err(Error::NotFound {
            msg: format!("Legal consultation with id={} not found", id),
        })
    }
}

#[ic_cdk::update]
fn add_legal_advisor(payload: LegalAdvisorPayload) -> Result<LegalAdvisor, Error> {
    validate_legal_advisor_payload(&payload)?;
    let id = ID_COUNTER
        .with(|counter| {
            let current_value = *counter.borrow().get();
            counter.borrow_mut().set(current_value + 1)
        })
        .expect("Cannot increment id counter");

    let advisor = LegalAdvisor {
        id,
        principal_string: caller().to_string(),
        name: payload.name,
        credentials: payload.credentials,
        rating: payload.rating,
    };

    do_insert_legal_advisor(&advisor);
    Ok(advisor)
}

#[ic_cdk::query]
fn get_legal_advisor(id: u64) -> Result<LegalAdvisor, Error> {
    match _get_legal_advisor(&id) {
        Some(advisor) => Ok(advisor),
        None => Err(Error::NotFound {
            msg: format!("Legal advisor with id={} not found", id),
        }),
    }
}

fn do_insert_legal_advisor(advisor: &LegalAdvisor) {
    LEGAL_ADVISORS.with(|service| service.borrow_mut().insert(advisor.id, advisor.clone()));
}

fn _get_legal_advisor(id: &u64) -> Option<LegalAdvisor> {
    LEGAL_ADVISORS.with(|service| service.borrow().get(id))
}

#[ic_cdk::update]
fn mark_consultation_as_completed(id: u64) -> Result<(), Error> {
    if let Some(consultation) = _get_legal_consultation(&id) {
        let advisor = get_legal_advisor(consultation.advisor_id)?;
        is_caller_advisor(&advisor)?;
        let mut updated_consultation = consultation.clone();
        updated_consultation.is_completed = true;
        LEGAL_CONSULTATIONS.with(|service| service.borrow_mut().insert(id, updated_consultation));
        Ok(())
    } else {
        Err(Error::NotFound {
            msg: format!("Legal consultation with id={} not found", id),
        })
    }
}

#[ic_cdk::update]
fn close_legal_consultation(id: u64, closed_at: u64) -> Result<(), Error> {
    if let Some(mut consultation) = _get_legal_consultation(&id) {
        let advisor = get_legal_advisor(consultation.advisor_id)?;
        is_caller_advisor(&advisor)?;
        consultation.closed_at = Some(closed_at);
        LEGAL_CONSULTATIONS.with(|service| service.borrow_mut().insert(id, consultation));
        Ok(())
    } else {
        Err(Error::NotFound {
            msg: format!("Legal consultation with id={} not found", id),
        })
    }
}

#[ic_cdk::query]
fn list_all_legal_consultations() -> Vec<LegalConsultation> {
    LEGAL_CONSULTATIONS.with(|service| {
        let map_ref = service.borrow();
        map_ref.iter().map(|(_, v)| v.clone()).collect()
    })
}

#[ic_cdk::query]
fn list_all_legal_advisors() -> Vec<LegalAdvisor> {
    LEGAL_ADVISORS.with(|service| {
        let map_ref = service.borrow();
        map_ref.iter().map(|(_, v)| v.clone()).collect()
    })
}

#[ic_cdk::update]
fn update_legal_consultation(
    id: u64,
    payload: LegalConsultationPayload
) -> Result<LegalConsultation, Error> {
    if let Some(consultation) = _get_legal_consultation(&id) {
        validate_legal_consultation_payload(&payload)?;
        let advisor = get_legal_advisor(consultation.advisor_id)?;
        is_caller_advisor(&advisor)?;

        let updated_consultation = LegalConsultation{
            id: consultation.id,
            advisor_id: payload.advisor_id,
            created_at: consultation.created_at.clone(),
            client_details: payload.client_details,
            closed_at: consultation.closed_at.clone(),
            details: payload.details,
            is_completed: consultation.is_completed
        };

        // Update the consultation in the map
        LEGAL_CONSULTATIONS.with(|service| service.borrow_mut().insert(id, consultation));
        Ok(updated_consultation)
    } else {
        Err(Error::NotFound {
            msg: format!("Legal consultation with id={} not found", id),
        })
    }
}

// Helper function to check whether the caller is the principal of the advisor
fn is_caller_advisor(advisor: &LegalAdvisor) -> Result<(), Error>{
    if advisor.principal_string != caller().to_string(){
        return Err(Error::NotAdviser { msg: format!("Caller is not the principal of the advisor") })
    }else{
        Ok(())
    }
}
// Helper function that return a bool value on whether the trimmed string is empty
fn is_invalid_string(str: &String) -> bool{
    str.trim().is_empty()

}
// Helper function to validate the input payload when creating or updating a consultation
fn validate_legal_consultation_payload(payload: &LegalConsultationPayload) -> Result<(), Error>{
    let mut errors: Vec<String> = Vec::new();
    if is_invalid_string(&payload.details){
        errors.push(format!("Consultation details='{}' cannot be empty.", payload.details))
    }
    if is_invalid_string(&payload.client_details.name){
        errors.push(format!("Client's name='{}' cannot be empty.", payload.client_details.name))
    }
    if is_invalid_string(&payload.client_details.email) {
        errors.push(format!("Client's email='{}' cannot be empty.", payload.client_details.email))
    }
    if errors.is_empty(){
        Ok(())
    }else{
        return Err(Error::InvalidPayload { errors })
    }
}
// Helper function to validate the input payload when creating or updating an advisor
fn validate_legal_advisor_payload(payload: &LegalAdvisorPayload) -> Result<(), Error>{
    let mut errors: Vec<String> = Vec::new();
    if is_invalid_string(&payload.name){
        errors.push(format!("Advisor name='{}' cannot be empty.", payload.name))
    }
    if is_invalid_string(&payload.credentials){
        errors.push(format!("Advisor credentials='{}' cannot be empty.", payload.credentials))
    }

    if errors.is_empty(){
        Ok(())
    }else{
        return Err(Error::InvalidPayload { errors })
    }
}

#[derive(candid::CandidType, Deserialize, Serialize)]
enum Error {
    NotFound { msg: String },
    NotAdviser{msg: String},
    InvalidPayload{errors: Vec<String>}
}

ic_cdk::export_candid!();
