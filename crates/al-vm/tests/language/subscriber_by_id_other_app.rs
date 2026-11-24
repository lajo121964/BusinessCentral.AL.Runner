//! A subscriber that writes its publisher as a bare id, in a dependency app, to OnAfterInsertEvent of a table of another app,
//! or to an IntegrationEvent of a codeunit of another app (assumption `subscriber-by-id-other-app-publisher`, al-vm events.rs `other_app_publisher`). Every expected
//! value is what BC 29.0.54011.55616 answered to the partner probe of that assumption: the by-id subscriber receives the event
//! as often as its twin that names the table, for Insert() and Insert(false), with RunTrigger No. App "base" stands in for
//! the table's app (Base Application in the probe), app "dep" for the subscriber's.

use al_builtins::assumptions::{Assumption, clear_hits, take_hits};
use al_builtins::native::Signal;
use al_sema::workspace::{AppKind, AppSource};

const BASE: &str = r#"
table 50230 "SBI Customer" { fields { field(1; "No."; Code[20]) { } } keys { key(PK; "No.") { } } }
"#;

/// The subscribers of the dependency app; `{extra}` adds one more.
fn dep(extra: &str) -> String {
    format!(
        r#"
codeunit 50231 "SBI Dep Subscribers"
{{
    SingleInstance = true;
    var ById: Integer; ByName: Integer; RunTriggers: Text;

    [EventSubscriber(ObjectType::Table, 50230, 'OnAfterInsertEvent', '', false, false)]
    local procedure InsertedById(var Rec: Record "SBI Customer"; RunTrigger: Boolean)
    begin
        ById += 1;
        RunTriggers += Format(RunTrigger) + ',';
    end;

    [EventSubscriber(ObjectType::Table, Database::"SBI Customer", 'OnAfterInsertEvent', '', false, false)]
    local procedure InsertedByName(var Rec: Record "SBI Customer"; RunTrigger: Boolean)
    begin
        ByName += 1;
    end;

    procedure Describe(): Text
    begin
        exit(StrSubstNo('byid=%1 byname=%2 runtrigger=%3', ById, ByName, RunTriggers));
    end;
}}
{extra}
"#
    )
}

const TEST: &str = r#"
codeunit 50239 T
{
    procedure Insert(): Text
    var
        Customer: Record "SBI Customer";
        Subscribers: Codeunit "SBI Dep Subscribers";
    begin
        Customer."No." := 'C1';
        Customer.Insert();
        Customer."No." := 'C2';
        Customer.Insert(false);
        exit(Subscribers.Describe());
    end;

    procedure Modify(): Text
    var
        Customer: Record "SBI Customer";
    begin
        Customer."No." := 'C1';
        Customer.Insert();
        Customer.Modify();
        exit('modified');
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

fn run(dep_extra: &str, routine: &str) -> (Result<String, Signal>, Vec<Assumption>) {
    run_with(dep_extra, TEST, routine)
}

fn run_with(dep_extra: &str, test: &str, routine: &str) -> (Result<String, Signal>, Vec<Assumption>) {
    let apps = vec![
        app("base", &[], BASE),
        app("dep", &["base"], &dep(dep_extra)),
        app("t", &["base", "dep"], test),
    ];
    let kinds = vec![AppKind::Dependency, AppKind::Dependency, AppKind::Workspace];
    let (ws, l) = al_sema::lower::compile_apps_with(apps, kinds, &Default::default(), None, Default::default());
    assert!(ws.diagnostics.is_empty(), "{:?}", ws.diagnostics);
    let mut b = al_testkit::Built::new(l);
    clear_hits();
    let r = b.run_in("T", routine).map(|v| al_testkit::text(Ok(v)));
    (r, take_hits())
}

/// The probe's cell E01.
#[test]
fn a_dependency_apps_by_id_subscriber_receives_another_apps_table_insert() {
    let (r, hits) = run("", "Insert");
    assert_eq!(r.unwrap(), "byid=2 byname=2 runtrigger=No,No,");
    assert!(hits.contains(&Assumption::SubscriberByIdOtherAppPublisher), "{hits:?}");
}

const PUB: &str = r#"
codeunit 50240 "SBI Pub"
{
    procedure Raise(Code: Code[20]) begin OnAfterThing(Code); end;
    [IntegrationEvent(false, false)]
    local procedure OnAfterThing(var Code: Code[20]) begin end;
    [BusinessEvent(false)]
    local procedure OnBusiness(var Code: Code[20]) begin end;
    procedure RaiseBusiness(Code: Code[20]) begin OnBusiness(Code); end;
}
"#;

/// The subscribers of the dependency app to the publisher app's codeunit, written by id and by name; `event` names the event.
fn codeunit_sub(event: &str) -> String {
    format!(
        r#"
codeunit 50241 "SBI CU Sub"
{{
    SingleInstance = true;
    var Log: Text;
    [EventSubscriber(ObjectType::Codeunit, 50240, '{event}', '', false, false)]
    local procedure ById(var Code: Code[20]) begin Log += 'id:' + Code + ';'; end;
    [EventSubscriber(ObjectType::Codeunit, Codeunit::"SBI Pub", '{event}', '', false, false)]
    local procedure ByName(var Code: Code[20]) begin Log += 'name:' + Code + ';'; end;
    procedure Take() R: Text begin R := Log; Log := ''; end;
}}
"#
    )
}

const RAISE: &str = r#"
codeunit 50249 T
{
    procedure Raise(): Text
    var Pub: Codeunit "SBI Pub"; Sub: Codeunit "SBI CU Sub";
    begin
        Pub.Raise('A');
        Pub.Raise('B');
        exit(Sub.Take());
    end;
    procedure RaiseBusiness(): Text
    var Pub: Codeunit "SBI Pub"; Sub: Codeunit "SBI CU Sub";
    begin
        Pub.RaiseBusiness('A');
        exit(Sub.Take());
    end;
}
"#;

fn run_codeunit(event: &str, routine: &str) -> (Result<String, Signal>, Vec<Assumption>) {
    let apps = vec![
        app("pub", &[], PUB),
        app("sub", &["pub"], &codeunit_sub(event)),
        app("t", &["pub", "sub"], RAISE),
    ];
    let kinds = vec![AppKind::Dependency, AppKind::Dependency, AppKind::Workspace];
    let (ws, l) = al_sema::lower::compile_apps_with(apps, kinds, &Default::default(), None, Default::default());
    assert!(ws.diagnostics.is_empty(), "{:?}", ws.diagnostics);
    let mut b = al_testkit::Built::new(l);
    clear_hits();
    let r = b.run_in("T", routine).map(|v| al_testkit::text(Ok(v)));
    (r, take_hits())
}

/// The probe's cell C01: each raise once, the subscriber written by id first (by method name), with the argument.
#[test]
fn a_dependency_apps_by_id_subscriber_receives_another_apps_codeunit_integration_event() {
    let (r, hits) = run_codeunit("OnAfterThing", "Raise");
    assert_eq!(r.unwrap(), "id:A;name:A;id:B;name:B;");
    assert!(hits.contains(&Assumption::SubscriberByIdOtherAppPublisher), "{hits:?}");
}

/// A BusinessEvent of another app's codeunit by id was not measured: refused.
#[test]
fn a_business_event_of_another_apps_codeunit_keeps_the_refusal() {
    match run_codeunit("OnBusiness", "RaiseBusiness").0 {
        Err(Signal::Unsupported(why)) => assert!(why.contains("by the id 50240"), "{why}"),
        other => panic!("{other:?}"),
    }
}

/// Not measured, so the event still refuses: another table event, and a manual subscriber.
#[test]
fn other_events_and_manual_subscribers_keep_the_refusal() {
    let modify = r#"
codeunit 50232 "SBI Dep Modify"
{
    [EventSubscriber(ObjectType::Table, 50230, 'OnAfterModifyEvent', '', false, false)]
    local procedure ModifiedById(var Rec: Record "SBI Customer"; var xRec: Record "SBI Customer"; RunTrigger: Boolean) begin end;
}
"#;
    match run(modify, "Modify").0 {
        Err(Signal::Unsupported(why)) => assert!(why.contains("by the id 50230"), "{why}"),
        other => panic!("{other:?}"),
    }
    let manual = r#"
codeunit 50233 "SBI Dep Manual"
{
    EventSubscriberInstance = Manual;
    [EventSubscriber(ObjectType::Table, 50230, 'OnAfterInsertEvent', '', false, false)]
    local procedure InsertedById(var Rec: Record "SBI Customer"; RunTrigger: Boolean) begin end;
}
"#;
    let bound = r#"
codeunit 50239 T
{
    procedure InsertBound(): Text
    var
        Customer: Record "SBI Customer";
        Manual: Codeunit "SBI Dep Manual";
    begin
        BindSubscription(Manual);
        Customer."No." := 'C1';
        Customer.Insert();
        exit('inserted');
    end;
}
"#;
    match run_with(manual, bound, "InsertBound").0 {
        Err(Signal::Unsupported(why)) => assert!(why.contains("by the id 50230"), "{why}"),
        other => panic!("{other:?}"),
    }
}
