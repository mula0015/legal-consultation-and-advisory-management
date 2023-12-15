#[macro_use]
extern crate serde;
use candid::{Decode, Encode};
use ic_cdk::api::time;
use ic_stable_structures::memory_manager::{MemoryId, MemoryManager, VirtualMemory};
use ic_stable_structures::{BoundedStorable, Cell, DefaultMemoryImpl, StableBTreeMap, Storable};
use std::{borrow::Cow, cell::RefCell};

type Memory = VirtualMemory<DefaultMemoryImpl>;
type IdCell = Cell<u64, Memory>;

#[derive(candid::CandidType, Clone, Serialize, Deserialize, Default)]
struct LegalConsultation {
    id: u64,
    user_id: u64,
    advisor_id: u64,
    details: String,
    created_at: u64,
    closed_at: Option<u64>,
    is_completed: bool,
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
    credentials: String,
    rating: f32,
    is_available: bool,
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
#[derive(candid::CandidType, Clone, Serialize, Deserialize)]
struct FeedbackRecord {
    consultation_id: u64,
    feedback: String,
    timestamp: u64, // Unix timestamp
}
impl Storable for FeedbackRecord {
    fn to_bytes(&self) -> Cow<[u8]> {
        Cow::Owned(Encode!(self).unwrap())
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
    }
}

impl BoundedStorable for FeedbackRecord {
    const MAX_SIZE: u32 = 2048;
    const IS_FIXED_SIZE: bool = false;
}
#[derive(candid::CandidType, Clone, Serialize, Deserialize)]
struct TimelineEvent {
    event_id: u64,
    consultation_id: u64,
    description: String,
    timestamp: u64,
}
impl Storable for TimelineEvent {
    fn to_bytes(&self) -> Cow<[u8]> {
        Cow::Owned(Encode!(self).unwrap())
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
    }
}

impl BoundedStorable for TimelineEvent {
    const MAX_SIZE: u32 = 2048;
    const IS_FIXED_SIZE: bool = false;
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
    static FEEDBACK_STORAGE: RefCell<StableBTreeMap<u64, FeedbackRecord, Memory>> = RefCell::new(
        StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(14)))
    ));
    static TIMELINE_EVENTS_STORAGE: RefCell<StableBTreeMap<u64, TimelineEvent, Memory>> = RefCell::new(
        StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(15)))
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
fn initiate_legal_consultation(user_id: u64, advisor_id: u64, details: String) -> Option<LegalConsultation> {
    let id = ID_COUNTER
        .with(|counter| {
            let current_value = *counter.borrow().get();
            counter.borrow_mut().set(current_value + 1)
        })
        .expect("Cannot increment id counter");

    let consultation = LegalConsultation {
        id,
        user_id,
        advisor_id,
        details,
        created_at: time(),
        closed_at: None,
        is_completed: false,
    };

    do_insert_legal_consultation(&consultation);
    Some(consultation)
}

#[ic_cdk::update]
fn update_legal_advisor(id: u64, name: String, credentials: String, rating: f32) -> Option<LegalAdvisor> {
    let advisor = LegalAdvisor {
        id,
        name,
        credentials,
        rating,
        is_available: true,
        
    };

    do_update_legal_advisor(&advisor);
    Some(advisor)
}

fn do_update_legal_advisor(advisor: &LegalAdvisor) {
    LEGAL_ADVISORS.with(|service| service.borrow_mut().insert(advisor.id, advisor.clone()));
}

fn do_insert_legal_consultation(consultation: &LegalConsultation) {
    LEGAL_CONSULTATIONS.with(|service| service.borrow_mut().insert(consultation.id, consultation.clone()));
}

fn _get_legal_consultation(id: &u64) -> Option<LegalConsultation> {
    LEGAL_CONSULTATIONS.with(|service| service.borrow().get(id))
}

#[ic_cdk::update]
fn delete_legal_consultation(id: u64) -> Result<(), Error> {
    if let Some(_) = _get_legal_consultation(&id) {
        LEGAL_CONSULTATIONS.with(|service| service.borrow_mut().remove(&id));
        Ok(())
    } else {
        Err(Error::NotFound {
            msg: format!("Legal consultation with id={} not found", id),
        })
    }
}

#[ic_cdk::update]
fn add_legal_advisor(name: String, credentials: String, rating: f32) -> Option<LegalAdvisor> {
    let id = ID_COUNTER
        .with(|counter| {
            let current_value = *counter.borrow().get();
            counter.borrow_mut().set(current_value + 1)
        })
        .expect("Cannot increment id counter");

    let advisor = LegalAdvisor {
        id,
        name,
        credentials,
        rating,
        is_available: true, // Set a default value for is_available
    };

    do_insert_legal_advisor(&advisor);
    Some(advisor)
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
fn search_consultations_by_user(user_id: u64) -> Vec<LegalConsultation> {
    LEGAL_CONSULTATIONS.with(|service| {
        service.borrow()
            .iter()
            .filter(|(_, consultation)| consultation.user_id == user_id)
            .map(|(_, consultation)| consultation.clone())
            .collect()
    })
}
#[ic_cdk::query]
fn generate_consultation_report(consultation_id: u64) -> Result<String, Error> {
    match _get_legal_consultation(&consultation_id) {
        Some(consultation) => {
            let report = format!(
                "Consultation Report\nID: {}\nUser ID: {}\nAdvisor ID: {}\nDetails: {}\nCreated At: {}\nClosed At: {:?}\nIs Completed: {}\n",
                consultation.id,
                consultation.user_id,
                consultation.advisor_id,
                consultation.details,
                consultation.created_at,
                consultation.closed_at,
                consultation.is_completed
            );
            Ok(report)
        },
        None => Err(Error::NotFound {
            msg: format!("Consultation with id={} not found", consultation_id),
        }),
    }
}
#[ic_cdk::update]
fn update_advisor_availability(advisor_id: u64, is_available: bool) -> Result<(), Error> {
    LEGAL_ADVISORS.with(|storage| {
        if let Some(mut advisor) = storage.borrow_mut().get(&advisor_id) {
            advisor.is_available = is_available; // Assuming 'is_available' field exists
            storage.borrow_mut().insert(advisor_id, advisor);
            Ok(())
        } else {
            Err(Error::NotFound {
                msg: format!("Advisor with id={} not found", advisor_id),
            })
        }
    })
}
fn get_timeline_events(consultation_id: u64) -> Vec<String> {
    TIMELINE_EVENTS_STORAGE.with(|storage| {
        storage.borrow()
            .iter()
            .filter(|(_, event)| event.consultation_id == consultation_id)
            .map(|(_, event)| format!("{}: {}", event.timestamp, event.description))
            .collect()
    })
}

#[ic_cdk::query]
fn track_consultation_timeline(consultation_id: u64) -> Result<Vec<String>, Error> {
    // Assuming there is a way to track and store timeline events
    let timeline_events = get_timeline_events(consultation_id); // Placeholder function
    if timeline_events.is_empty() {
        Err(Error::NotFound {
            msg: format!("No timeline events found for consultation with id={}", consultation_id),
        })
    } else {
        Ok(timeline_events)
    }
}
#[ic_cdk::update]
fn collect_consultation_feedback(consultation_id: u64, feedback: String) -> Result<(), Error> {
    let feedback_record = FeedbackRecord {
        consultation_id,
        feedback,
        timestamp: ic_cdk::api::time(),
    };

    FEEDBACK_STORAGE.with(|storage| {
        let feedback_id = generate_feedback_id(); // Function to generate unique feedback ID
        storage.borrow_mut().insert(feedback_id, feedback_record);
    });

    Ok(())
}
fn generate_feedback_id() -> u64 {
    ic_cdk::api::time()
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
    user_id: Option<u64>,
    advisor_id: Option<u64>,
    details: Option<String>,
    is_completed: Option<bool>,
) -> Result<(), Error> {
    if let Some(mut consultation) = _get_legal_consultation(&id) {
        // Update fields if provided
        if let Some(user_id) = user_id {
            consultation.user_id = user_id;
        }
        if let Some(advisor_id) = advisor_id {
            consultation.advisor_id = advisor_id;
        }
        if let Some(details) = details {
            consultation.details = details;
        }
        if let Some(is_completed) = is_completed {
            consultation.is_completed = is_completed;
        }

        // Update the consultation in the map
        LEGAL_CONSULTATIONS.with(|service| service.borrow_mut().insert(id, consultation));
        Ok(())
    } else {
        Err(Error::NotFound {
            msg: format!("Legal consultation with id={} not found", id),
        })
    }
}

#[derive(candid::CandidType, Deserialize, Serialize)]
enum Error {
    NotFound { msg: String },
}

ic_cdk::export_candid!();