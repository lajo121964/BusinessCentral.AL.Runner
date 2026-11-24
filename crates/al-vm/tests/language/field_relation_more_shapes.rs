//! Field.RelationFieldNo of a TableRelation that names a table and no field, in the shapes beyond the pin
//! (metadata/platform-tables/FieldRelationFieldNo.al) that assumption `relation-field-no-without-field-more-shapes` covers
//! (al-vm platform/field_relation.rs `no_field_shape`). Every expected value is what BC 29.0.54011.55616 answered to the
//! partner probe of that assumption: RelationFieldNo 0, RelationTableNo the target, for each shape; a relation that names the
//! field gives its number. App "base" stands in for Base Application (another app than the test app) and declares the platform
//! tables Field and Company by their field lists, as the System app does (Options as Integer).

use al_builtins::assumptions::{Assumption, clear_hits, take_hits};
use al_builtins::native::Signal;
use al_builtins::value::Value;
use al_sema::workspace::{AppKind, AppSource};

const BASE: &str = r#"
table 2000000041 "Field"
{
    fields
    {
        field(1; TableNo; Integer) { } field(2; "No."; Integer) { } field(3; TableName; Text[30]) { }
        field(4; FieldName; Text[30]) { } field(5; Type; Option) { OptionMembers = {OPT}; } field(6; Len; Integer) { } field(7; Class; Option) { OptionMembers = {OPT}; }
        field(8; Enabled; Boolean) { } field(9; "Type Name"; Text[30]) { } field(10; ExternalName; Text[100]) { }
        field(20; "Field Caption"; Text[80]) { } field(21; RelationTableNo; Integer) { } field(22; RelationFieldNo; Integer) { }
        field(23; SQLDataType; Option) { OptionMembers = {OPT}; } field(24; OptionString; Text[2047]) { } field(25; ObsoleteState; Option) { OptionMembers = {OPT}; }
        field(26; ObsoleteReason; Text[248]) { } field(27; DataClassification; Option) { OptionMembers = {OPT}; } field(28; IsPartOfPrimaryKey; Boolean) { }
        field(60; "App Package ID"; Guid) { } field(61; "App Runtime Package ID"; Guid) { } field(62; OptimizeForTextSearch; Boolean) { }
        field(63; Access; Option) { OptionMembers = {OPT}; } field(64; IsAllowedInCustomizations; Boolean) { }
    }
    keys { key(PK; TableNo, "No.") { } }
}
table 2000000006 Company
{
    DataPerCompany = false;
    fields
    {
        field(1; Name; Text[30]) { } field(2; "Evaluation Company"; Boolean) { } field(3; "Display Name"; Text[250]) { }
        field(8000; Id; Guid) { } field(8005; "Business Profile Id"; Text[250]) { }
    }
    keys { key(PK; Name) { } }
}
table 50200 "RFN Vendor" { fields { field(1; "No."; Code[20]) { } field(2; Name; Text[100]) { } } keys { key(PK; "No.") { } } }
table 50201 "RFN Currency" { fields { field(1; Code; Code[10]) { } } keys { key(PK; Code) { } } }
table 50202 "RFN Reason Code" { fields { field(1; Code; Code[10]) { } } keys { key(PK; Code) { } } }
table 50203 "RFN Two Key" { fields { field(1; A; Code[10]) { } field(2; B; Code[10]) { } } keys { key(PK; A, B) { } } }
"#;

const TEST: &str = r#"
table 50210 "RFN User" { fields { field(1; "User ID"; Code[50]) { } field(2; Approver; Code[50]) { } } keys { key(PK; "User ID") { } } }
table 50211 "RFN Src"
{
    fields
    {
        field(1; "Entry No."; Integer) { }
        field(2; "Vendor No."; Code[20]) { TableRelation = "RFN Vendor"; }
        field(3; "Currency Code"; Code[10]) { TableRelation = "RFN Currency"; }
        field(4; "Vendor Filtered"; Code[20]) { TableRelation = "RFN Vendor" where(Name = filter(<> '')); }
        field(5; "Own User Filtered"; Code[50]) { TableRelation = "RFN User" where(Approver = filter(<> '')); }
        field(6; "Vendor No. Named"; Code[20]) { TableRelation = "RFN Vendor"."No."; }
        field(7; "Vendor Const"; Code[20]) { TableRelation = "RFN Vendor" where(Name = const('X')); }
        field(8; "Two Key"; Code[10]) { TableRelation = "RFN Two Key"; }
    }
    keys { key(PK; "Entry No.") { } }
}
table 50212 "RFN Reason"
{
    fields { field(1; "Table ID"; Integer) { } field(2; Type; Integer) { } field(3; "Reason Code"; Code[10]) { TableRelation = "RFN Reason Code"; } }
    keys { key(PK; "Table ID", Type, "Reason Code") { } }
}
table 50213 "RFN User Setup"
{
    fields { field(1; "User ID"; Code[50]) { TableRelation = "RFN User"; } field(2; Name; Text[50]) { } }
    keys { key(PK; "User ID") { } }
}
table 50214 "RFN Portal Company"
{
    fields { field(1; "Portal User"; Code[50]) { } field(2; "Company Name"; Text[30]) { TableRelation = Company; } field(3; "Other Company"; Text[30]) { TableRelation = Company; } }
    keys { key(PK; "Portal User", "Company Name") { } }
}
table 50217 "RFN Temp Src"
{
    TableType = Temporary;
    fields
    {
        field(1; "Entry No."; Integer) { }
        field(2; "To User Id"; Code[50]) { TableRelation = "RFN User" where(Approver = filter(<> '')); }
        field(3; "Vendor No."; Code[20]) { TableRelation = "RFN Vendor"; }
        field(4; "Ext User Id"; Code[50]) { TableRelation = "RFN User" where("Ext Flag" = filter(<> '')); }
    }
    keys { key(PK; "Entry No.") { } }
}
tableextension 50218 "RFN User Ext" extends "RFN User" { fields { field(50218; "Ext Flag"; Code[20]) { } } }
table 50215 "RFN Dimension" { fields { field(1; Code; Code[20]) { } } keys { key(PK; Code) { } } }
table 50216 "RFN Dimension Value"
{
    fields { field(1; "Dimension Code"; Code[20]) { TableRelation = "RFN Dimension"; } field(2; Code; Code[20]) { } }
    keys { key(PK; "Dimension Code", Code) { } }
}
codeunit 50219 T
{
    procedure Describe(TableNo: Integer; FromField: Integer; ToField: Integer): Text
    var
        Field: Record Field;
        Result: Text;
    begin
        Field.SetRange(TableNo, TableNo);
        Field.SetRange("No.", FromField, ToField);
        if Field.FindSet() then
            repeat
                Result += StrSubstNo('[%1 rt=%2 rf=%3]', Field.FieldName, Field.RelationTableNo, Field.RelationFieldNo);
            until Field.Next() = 0;
        exit(Result);
    end;
}
"#;

fn app(id: &str, deps: &[&str], src: &str) -> AppSource {
    AppSource {
        id: id.into(),
        name: format!("App {id}"),
        dependencies: deps.iter().map(|d| d.to_string()).collect(),
        files: vec![(format!("{id}.al"), src.into())],
    }
}

/// The fields `from..=to` of table `table` as `Describe` lists them, and the assumptions the read marked.
fn describe(table: i32, from: i32, to: i32) -> (Result<String, Signal>, Vec<Assumption>) {
    let members: Vec<String> = (0..64).map(|i| format!("M{i}")).collect();
    let base = BASE.replace("{OPT}", &members.join(","));
    let apps = vec![app("base", &[], &base), app("t", &["base"], TEST)];
    // Both compiled whole: a dependency app compiles on demand, and nothing here uses its tables but the relations.
    let kinds = vec![AppKind::Workspace, AppKind::Workspace];
    let (ws, l) = al_sema::lower::compile_apps_with(apps, kinds, &Default::default(), None, Default::default());
    assert!(ws.diagnostics.is_empty(), "{:?}", ws.diagnostics);
    let mut b = al_testkit::Built::new(l);
    let (oi, ri) = b.routine_in("T", "Describe");
    clear_hits();
    let r = b
        .call(
            oi,
            ri,
            vec![Value::Integer(table), Value::Integer(from), Value::Integer(to)],
        )
        .map(|(v, _)| al_testkit::text(Ok(v)));
    (r, take_hits())
}

fn assumed(table: i32, from: i32, to: i32) -> String {
    let (r, hits) = describe(table, from, to);
    assert!(
        hits.contains(&Assumption::RelationFieldNoWithoutFieldMoreShapes),
        "{hits:?}"
    );
    r.unwrap()
}

fn refused(table: i32, field: i32) -> String {
    match describe(table, field, field).0 {
        Err(Signal::Unsupported(why)) => why.to_string(),
        other => panic!("{other:?}"),
    }
}

/// Probe cell F01: Code[20] and Code[10] onto tables of another app, a filter where onto another app's table and onto
/// the test app's own; the field named gives its number.
#[test]
fn a_code_relation_onto_another_apps_table_or_with_a_filter_where_reads_zero() {
    assert_eq!(
        assumed(50211, 2, 6),
        "[Vendor No. rt=50200 rf=0][Currency Code rt=50201 rf=0][Vendor Filtered rt=50200 rf=0]\
         [Own User Filtered rt=50210 rf=0][Vendor No. Named rt=50200 rf=1]"
    );
}

/// Probe cells F02 (another app, the last of three key fields), F03 (the own app, the whole key) and F05 (Dimension
/// Value."Dimension Code": the own app, the first of two key fields).
#[test]
fn a_code_relation_of_a_key_field_reads_zero() {
    assert_eq!(assumed(50212, 3, 3), "[Reason Code rt=50202 rf=0]");
    assert_eq!(assumed(50213, 1, 1), "[User ID rt=50210 rf=0]");
    assert_eq!(assumed(50216, 1, 1), "[Dimension Code rt=50215 rf=0]");
}

/// Probe cell F04: Text[30] onto Company, in the key and outside it.
#[test]
fn a_text_relation_onto_company_reads_zero() {
    assert_eq!(
        assumed(50214, 2, 3),
        "[Company Name rt=2000000006 rf=0][Other Company rt=2000000006 rf=0]"
    );
}

/// Probe cell F06: a Temporary source, with a filter where on the target's field and on a table extension's.
#[test]
fn a_relation_of_a_temporary_source_reads_zero() {
    assert_eq!(
        assumed(50217, 2, 4),
        "[To User Id rt=50210 rf=0][Vendor No. rt=50200 rf=0][Ext User Id rt=50210 rf=0]"
    );
}

/// Not measured, so a read of RelationFieldNo still refuses: a const where onto another app's table, and a target keyed by
/// two Code fields in another app.
#[test]
fn unmeasured_shapes_keep_the_refusal() {
    for field in [7, 8] {
        let why = refused(50211, field);
        assert!(why.contains("RelationFieldNo") && why.contains("not measured"), "{why}");
    }
}
