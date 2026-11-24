SetrangeTextIntoGuidSystemidOrTemporary => {
    id: "setrange-text-into-guid-systemid-or-temporary",
    assumed: "FieldRef.SetRange(X) with one typed non-empty Text or Code X into a Guid field, in an en-US session and filter group 0, answers as the earlier conversion does (a text that holds a Guid, with or without braces, filters on it; a text that is no Guid raises InvalidGuidFormat) also on the SystemId field of a record that is not temporary and on a stored Guid field of a temporary record (al-vm setrange_convert.rs `text_into_guid_other_shape`), beyond the stored field of a table that is not temporary that assumption setrange-other-type-convert covers. Seen on BC 29.0.54011.55616 with a partner probe: RecordRef.Field(SystemId).SetRange of the Format text of a row's SystemId (braces, upper case) and of it without braces in lower case each found that row alone; SetRange('ABC') there raised InvalidGuidFormat (`Invalid format of GUID string. ...`); on a temporary record a stored Guid field's SetRange of the Format text of its value found the row. Not covered and refused: other system fields, two values, SetFilter, a Variant, another language or filter group.",
    tests: "Continia Document Capture Test Suite (all of it): CDC Record ID Mgt.FilterRecRef calls FieldRef.SetRange(FieldFilter) with a Text[250] holding a Guid during the test install.",
    probe: "A table with a Code key and a Guid field; on a RecordRef of it FieldRef.SetRange of the Format text of a row's SystemId, with and without braces, then Count and FindFirst; SetRange('ABC') on SystemId under asserterror, with GetLastErrorText and GetLastErrorCode; on a temporary record, SetRange of the Format text of the Guid field's value, then Count and FindFirst. Measured as a partner probe; the corpus probe (the file below) is still to be written. Still wanted: the other system fields, SystemId of a temporary record, BC 27 and 28.",
    awaiting: Awaiting {
        issues: &[],
        files: &["tests/al-language/recordref-fieldref/fieldref/SrcSetRangeGuidSystemId.al"],
        since: "2026-11-24",
    },
}
