//! `FieldRef.SetRange(X)` of one typed Text into a Guid field in the shapes beyond the measured one that assumption
//! `setrange-text-into-guid-systemid-or-temporary` covers (al-vm setrange_convert.rs `text_into_guid_other_shape`). Every
//! expected value is what BC 29.0.54011.55616 answered to the partner probe of that assumption (each test names its cell).

use al_builtins::assumptions::{Assumption, clear_hits, take_hits};
use al_builtins::native::Signal;
use al_testkit::Built;

const SRC: &str = r#"
table 50950 "ALR Guid Row"
{
    fields { field(1; "No."; Code[20]) { } field(2; Ref; Guid) { } }
    keys { key(PK; "No.") { Clustered = true; } }
}
codeunit 50951 C
{
    local procedure Found(var RecRef: RecordRef): Text
    begin
        if not RecRef.FindFirst() then
            exit('none');
        exit(Format(RecRef.Field(1).Value));
    end;

    procedure SystemIdText(): Text
    var Row: Record "ALR Guid Row"; RecRef: RecordRef; FieldRef: FieldRef; Filter: Text[250]; Result: Text;
    begin
        Row."No." := 'G1'; Row.Insert();
        Row."No." := 'G2'; Row.Insert();
        Row.Get('G2');
        RecRef.Open(Database::"ALR Guid Row");
        FieldRef := RecRef.Field(Row.FieldNo(SystemId));
        Filter := Format(Row.SystemId);
        FieldRef.SetRange(Filter);
        Result := StrSubstNo('braces count=%1 first=%2', RecRef.Count(), Found(RecRef));
        Filter := LowerCase(DelChr(Format(Row.SystemId), '=', '{}'));
        FieldRef.SetRange(Filter);
        exit(Result + StrSubstNo(' bare count=%1 first=%2', RecRef.Count(), Found(RecRef)));
    end;

    procedure SystemIdNotAGuid(): Text
    var RecRef: RecordRef; FieldRef: FieldRef; Filter: Text[250];
    begin
        RecRef.Open(Database::"ALR Guid Row");
        FieldRef := RecRef.Field(2000000000);
        Filter := 'ABC';
        asserterror FieldRef.SetRange(Filter);
        exit(StrSubstNo('err=[%1] code=[%2]', GetLastErrorText(), GetLastErrorCode()));
    end;

    procedure TemporaryStoredGuid(): Text
    var Row: Record "ALR Guid Row" temporary; RecRef: RecordRef; FieldRef: FieldRef; Filter: Text[250]; Id: Guid;
    begin
        Id := CreateGuid();
        Row."No." := 'T1'; Row.Ref := Id; Row.Insert();
        RecRef.GetTable(Row);
        FieldRef := RecRef.Field(2);
        Filter := Format(Id);
        FieldRef.SetRange(Filter);
        exit(StrSubstNo('temp count=%1 first=%2', RecRef.Count(), Found(RecRef)));
    end;

    procedure SystemIdTwoValues(): Text
    var Row: Record "ALR Guid Row"; RecRef: RecordRef; FieldRef: FieldRef; Filter: Text[250];
    begin
        Row."No." := 'G1'; Row.Insert();
        RecRef.Open(Database::"ALR Guid Row");
        FieldRef := RecRef.Field(2000000000);
        Filter := Format(Row.SystemId);
        FieldRef.SetRange(Filter, Filter);
        exit(Format(RecRef.Count()));
    end;
}
"#;

fn run(routine: &str) -> (Result<String, Signal>, Vec<Assumption>) {
    let mut b = Built::from_source(SRC);
    clear_hits();
    let r = b.run_in("C", routine).map(|v| al_testkit::text(Ok(v)));
    (r, take_hits())
}

fn assumed(routine: &str) -> String {
    let (r, hits) = run(routine);
    assert!(
        hits.contains(&Assumption::SetrangeTextIntoGuidSystemidOrTemporary),
        "{routine}: {hits:?}"
    );
    r.unwrap()
}

/// Probe cell G01.
#[test]
fn a_text_holding_a_guid_filters_the_system_id_field() {
    assert_eq!(assumed("SystemIdText"), "braces count=1 first=G2 bare count=1 first=G2");
}

/// Probe cell G02.
#[test]
fn a_text_that_is_no_guid_raises_invalid_guid_format_on_system_id() {
    assert_eq!(
        assumed("SystemIdNotAGuid"),
        "err=[Invalid format of GUID string. The correct format of the GUID string is: CDEF7890-ABCD-0123-1234-567890ABCDEF \
         where 0-9, A-F symbolizes hexadecimal digits.] code=[InvalidGuidFormat]"
    );
}

/// Probe cell G03.
#[test]
fn a_text_holding_a_guid_filters_a_stored_guid_field_of_a_temporary_record() {
    assert_eq!(assumed("TemporaryStoredGuid"), "temp count=1 first=T1");
}

/// Two values were not measured on SystemId: refused.
#[test]
fn two_values_on_system_id_keep_the_refusal() {
    match run("SystemIdTwoValues").0 {
        Err(Signal::Unsupported(why)) => assert!(why.contains("into a Guid field"), "{why}"),
        other => panic!("{other:?}"),
    }
}
