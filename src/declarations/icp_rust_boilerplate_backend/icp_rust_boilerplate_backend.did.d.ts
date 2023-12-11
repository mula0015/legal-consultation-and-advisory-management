import type { Principal } from '@dfinity/principal';
import type { ActorMethod } from '@dfinity/agent';

export type Error = { 'NotFound' : { 'msg' : string } };
export interface LegalAdvisor {
  'id' : bigint,
  'name' : string,
  'credentials' : string,
  'rating' : number,
}
export interface LegalConsultation {
  'id' : bigint,
  'closed_at' : [] | [bigint],
  'created_at' : bigint,
  'user_id' : bigint,
  'is_completed' : boolean,
  'details' : string,
  'advisor_id' : bigint,
}
export type Result = { 'Ok' : null } |
  { 'Err' : Error };
export type Result_1 = { 'Ok' : LegalAdvisor } |
  { 'Err' : Error };
export type Result_2 = { 'Ok' : LegalConsultation } |
  { 'Err' : Error };
export interface _SERVICE {
  'add_legal_advisor' : ActorMethod<
    [string, string, number],
    [] | [LegalAdvisor]
  >,
  'close_legal_consultation' : ActorMethod<[bigint, bigint], Result>,
  'delete_legal_consultation' : ActorMethod<[bigint], Result>,
  'get_legal_advisor' : ActorMethod<[bigint], Result_1>,
  'get_legal_consultation' : ActorMethod<[bigint], Result_2>,
  'initiate_legal_consultation' : ActorMethod<
    [bigint, bigint, string],
    [] | [LegalConsultation]
  >,
  'list_all_legal_advisors' : ActorMethod<[], Array<LegalAdvisor>>,
  'list_all_legal_consultations' : ActorMethod<[], Array<LegalConsultation>>,
  'mark_consultation_as_completed' : ActorMethod<[bigint], Result>,
  'update_legal_advisor' : ActorMethod<
    [bigint, string, string, number],
    [] | [LegalAdvisor]
  >,
  'update_legal_consultation' : ActorMethod<
    [bigint, [] | [bigint], [] | [bigint], [] | [string], [] | [boolean]],
    Result
  >,
}
