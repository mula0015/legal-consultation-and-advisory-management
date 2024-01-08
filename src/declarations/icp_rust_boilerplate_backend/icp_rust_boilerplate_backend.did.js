export const idlFactory = ({ IDL }) => {
  const LegalAdvisor = IDL.Record({
    'id' : IDL.Nat64,
    'name' : IDL.Text,
    'credentials' : IDL.Text,
    'rating' : IDL.Float32,
  });
  const Error = IDL.Variant({ 'NotFound' : IDL.Record({ 'msg' : IDL.Text }) });
  const Result = IDL.Variant({ 'Ok' : IDL.Null, 'Err' : Error });
  const Result_1 = IDL.Variant({ 'Ok' : LegalAdvisor, 'Err' : Error });
  const LegalConsultation = IDL.Record({
    'id' : IDL.Nat64,
    'closed_at' : IDL.Opt(IDL.Nat64),
    'created_at' : IDL.Nat64,
    'is_completed' : IDL.Bool,
    'details' : IDL.Text,
    'advisor_id' : IDL.Nat64,
  });
  const Result_2 = IDL.Variant({ 'Ok' : LegalConsultation, 'Err' : Error });
  return IDL.Service({
    'add_legal_advisor' : IDL.Func(
        [IDL.Text, IDL.Text, IDL.Float32],
        [IDL.Opt(LegalAdvisor)],
        [],
      ),
    'close_legal_consultation' : IDL.Func([IDL.Nat64, IDL.Nat64], [Result], []),
    'delete_legal_consultation' : IDL.Func([IDL.Nat64], [Result], []),
    'get_legal_advisor' : IDL.Func([IDL.Nat64], [Result_1], ['query']),
    'get_legal_consultation' : IDL.Func([IDL.Nat64], [Result_2], ['query']),
    'initiate_legal_consultation' : IDL.Func(
        [IDL.Nat64, IDL.Text],
        [IDL.Opt(LegalConsultation)],
        [],
      ),
    'list_all_legal_advisors' : IDL.Func(
        [],
        [IDL.Vec(LegalAdvisor)],
        ['query'],
      ),
    'list_all_legal_consultations' : IDL.Func(
        [],
        [IDL.Vec(LegalConsultation)],
        ['query'],
      ),
    'mark_consultation_as_completed' : IDL.Func([IDL.Nat64], [Result], []),
    'update_legal_advisor' : IDL.Func(
        [IDL.Nat64, IDL.Text, IDL.Text, IDL.Float32],
        [IDL.Opt(LegalAdvisor)],
        [],
      ),
    'update_legal_consultation' : IDL.Func(
        [IDL.Nat64, IDL.Opt(IDL.Nat64), IDL.Opt(IDL.Text), IDL.Opt(IDL.Bool)],
        [Result],
        [],
      ),
  });
};
export const init = ({ IDL }) => { return []; };
