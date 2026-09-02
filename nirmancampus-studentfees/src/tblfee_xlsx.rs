//! Parse tblfee.xlsx and upsert rows by Receipt ID into MySQL `tblfee`.

use std::collections::BTreeMap;
use std::io::Cursor;

use calamine::{Data, Reader, Xlsx, open_workbook_from_rs};
use chrono::{NaiveDate, NaiveDateTime, NaiveTime, TimeDelta};
use sea_orm::{
    ActiveValue::Set, ColumnTrait, ConnectionTrait, EntityTrait, QueryFilter, TransactionTrait,
    sea_query::OnConflict,
};

use crate::entities::fee::{self, Entity as FeeEntity};
use crate::parse::{parse_date_flexible, parse_flag, parse_optional_text};

const BATCH_SIZE: usize = 20;

#[derive(Clone, Debug, Default)]
pub struct ParsedFeeRow {
    pub id: i32,
    pub adm_session: String,
    pub adm_year: String,
    pub dod: Option<NaiveDateTime>,
    pub submit: String,
    pub prog: String,
    pub enroll: String,
    pub student: String,
    pub year_sem: String,
    pub category: String,
    pub dob: String,
    pub contact: String,
    pub deposit: String,
    pub nsd: String,
    pub fee: String,
    pub courses: String,
    pub remarks: String,
    pub deposit_by: String,
    pub ts: String,
    pub medium: String,
    pub mother_name: String,
    pub father_name: String,
    pub username: String,
    pub control_id: String,
    pub descrepency: String,
    pub university: String,
    pub payment_mode: String,
    pub trans_id: String,
    pub bank: String,
    pub rm: String,
    pub is_reconciled: String,
    pub online_exported: String,
}

#[derive(Clone, Debug, Default)]
pub struct SyncReport {
    pub inserted: u64,
    pub updated: u64,
    pub skipped: u64,
}

pub fn excel_serial_to_date(serial: f64) -> Option<NaiveDate> {
    if !serial.is_finite() {
        return None;
    }
    let days = serial.trunc() as i64;
    NaiveDate::from_ymd_opt(1899, 12, 30)?.checked_add_signed(TimeDelta::days(days))
}

fn date_to_dod(d: NaiveDate) -> Option<NaiveDateTime> {
    Some(d.and_time(NaiveTime::MIN))
}

fn cell_to_string(cell: &Data) -> String {
    match cell {
        Data::Empty => String::new(),
        Data::String(s) => s.trim().to_string(),
        Data::Float(f) => {
            if f.fract() == 0.0 {
                format!("{}", *f as i64)
            } else {
                f.to_string()
            }
        }
        Data::Int(i) => i.to_string(),
        Data::Bool(b) => b.to_string(),
        Data::DateTime(dt) => {
            if let Some(d) = excel_serial_to_date(dt.as_f64()) {
                d.format("%d-%m-%Y").to_string()
            } else {
                dt.as_f64().to_string()
            }
        }
        Data::DateTimeIso(s) | Data::DurationIso(s) => s.clone(),
        Data::Error(err) => format!("{err:?}"),
    }
}

fn cell_to_dod(cell: &Data) -> Option<NaiveDateTime> {
    match cell {
        Data::Empty => None,
        Data::DateTime(dt) => excel_serial_to_date(dt.as_f64()).and_then(date_to_dod),
        Data::Float(f) => excel_serial_to_date(*f).and_then(date_to_dod),
        Data::Int(i) => excel_serial_to_date(*i as f64).and_then(date_to_dod),
        Data::String(s) => parse_date_flexible(s)
            .and_then(date_to_dod)
            .or_else(|| s.parse::<f64>().ok().and_then(excel_serial_to_date).and_then(date_to_dod)),
        Data::DateTimeIso(s) => parse_date_flexible(s).and_then(date_to_dod),
        _ => None,
    }
}

fn header_key(s: &str) -> String {
    s.trim().to_ascii_lowercase()
}

fn col_index(headers: &[String], name: &str) -> Option<usize> {
    let want = header_key(name);
    headers.iter().position(|h| header_key(h) == want)
}

fn cell_at(row: &[Data], idx: Option<usize>) -> String {
    idx.and_then(|i| row.get(i))
        .map(cell_to_string)
        .unwrap_or_default()
}

fn parse_id(row: &[Data], idx: Option<usize>) -> Option<i32> {
    let raw = cell_at(row, idx);
    let raw = raw.trim();
    if raw.is_empty() {
        return None;
    }
    if let Ok(n) = raw.parse::<i64>() {
        return i32::try_from(n).ok().filter(|n| *n > 0);
    }
    if let Ok(f) = raw.parse::<f64>() {
        let n = f.trunc() as i64;
        return i32::try_from(n).ok().filter(|n| *n > 0);
    }
    None
}

/// Read the first worksheet and return unique rows keyed by Receipt ID (last duplicate wins).
pub fn parse_tblfee_xlsx(bytes: &[u8]) -> Result<(Vec<ParsedFeeRow>, u64), String> {
    if bytes.is_empty() {
        return Err("empty file".into());
    }
    let mut workbook: Xlsx<_> = open_workbook_from_rs(Cursor::new(bytes.to_vec()))
        .map_err(|e| format!("open xlsx: {e}"))?;
    let names = workbook.sheet_names();
    let Some(name) = names.first().cloned() else {
        return Err("workbook has no sheets".into());
    };
    let range = workbook
        .worksheet_range(&name)
        .map_err(|e| format!("read sheet {name}: {e}"))?;
    let mut row_iter = range.rows();
    let Some(header_row) = row_iter.next() else {
        return Err("sheet has no header row".into());
    };
    let headers: Vec<String> = header_row.iter().map(cell_to_string).collect();
    let id_idx = col_index(&headers, "ID").ok_or_else(|| "missing ID column".to_string())?;
    let idx = |name: &str| col_index(&headers, name);

    let adm_session = idx("AdmSession");
    let adm_year = idx("AdmYear");
    let dod = idx("DOD");
    let submit = idx("Submit");
    let prog = idx("Prog");
    let enroll = idx("Enroll");
    let student = idx("Student");
    let year_sem = idx("YearSem");
    let category = idx("Category");
    let dob = idx("DOB");
    let contact = idx("Contact");
    let deposit = idx("Deposit");
    let nsd = idx("NSD");
    let fee = idx("Fee");
    let courses = idx("Courses");
    let remarks = idx("Remarks");
    let deposit_by = idx("DepositBy");
    let ts = idx("TS");
    let medium = idx("medium");
    let mother = idx("mother");
    let father = idx("father");
    let username = idx("username");
    let controlid = idx("controlid");
    let descrepency = idx("descrepency");
    let university = idx("University");
    let payment_mode = idx("PaymentMode");
    let trans_id = idx("TransID");
    let bank = idx("Bank");
    let rm = idx("RM");
    let is_reconciled = idx("IsReconciled");
    let online_exported = idx("OnlineExported");

    let mut skipped = 0u64;
    let mut by_id: BTreeMap<i32, ParsedFeeRow> = BTreeMap::new();
    for row in row_iter {
        if row.iter().all(|c| matches!(c, Data::Empty)) {
            continue;
        }
        let Some(id) = parse_id(row, Some(id_idx)) else {
            skipped += 1;
            continue;
        };
        let dod_val = dod.and_then(|i| row.get(i)).and_then(cell_to_dod);
        by_id.insert(
            id,
            ParsedFeeRow {
                id,
                adm_session: cell_at(row, adm_session),
                adm_year: cell_at(row, adm_year),
                dod: dod_val,
                submit: cell_at(row, submit),
                prog: cell_at(row, prog),
                enroll: cell_at(row, enroll),
                student: cell_at(row, student),
                year_sem: cell_at(row, year_sem),
                category: cell_at(row, category),
                dob: cell_at(row, dob),
                contact: cell_at(row, contact),
                deposit: cell_at(row, deposit),
                nsd: cell_at(row, nsd),
                fee: cell_at(row, fee),
                courses: cell_at(row, courses),
                remarks: cell_at(row, remarks),
                deposit_by: cell_at(row, deposit_by),
                ts: cell_at(row, ts),
                medium: cell_at(row, medium),
                mother_name: cell_at(row, mother),
                father_name: cell_at(row, father),
                username: cell_at(row, username),
                control_id: cell_at(row, controlid),
                descrepency: cell_at(row, descrepency),
                university: cell_at(row, university),
                payment_mode: cell_at(row, payment_mode),
                trans_id: cell_at(row, trans_id),
                bank: cell_at(row, bank),
                rm: cell_at(row, rm),
                is_reconciled: cell_at(row, is_reconciled),
                online_exported: cell_at(row, online_exported),
            },
        );
    }
    Ok((by_id.into_values().collect(), skipped))
}

fn to_active(row: &ParsedFeeRow) -> fee::ActiveModel {
    fee::ActiveModel {
        id: Set(row.id),
        adm_session: Set(parse_optional_text(&row.adm_session)),
        adm_year: Set(parse_optional_text(&row.adm_year)),
        dod: Set(row.dod),
        submit: Set(parse_optional_text(&row.submit)),
        prog: Set(parse_optional_text(&row.prog)),
        enroll: Set(parse_optional_text(&row.enroll)),
        student: Set(parse_optional_text(&row.student)),
        year_sem: Set(parse_optional_text(&row.year_sem)),
        category: Set(parse_optional_text(&row.category)),
        dob: Set(parse_optional_text(&row.dob)),
        contact: Set(parse_optional_text(&row.contact)),
        deposit: Set(parse_optional_text(&row.deposit)),
        nsd: Set(parse_optional_text(&row.nsd)),
        fee: Set(parse_optional_text(&row.fee)),
        courses: Set(parse_optional_text(&row.courses)),
        remarks: Set(parse_optional_text(&row.remarks)),
        deposit_by: Set(parse_optional_text(&row.deposit_by)),
        ts: Set(parse_optional_text(&row.ts)),
        medium: Set(parse_optional_text(&row.medium)),
        mother_name: Set(parse_optional_text(&row.mother_name)),
        father_name: Set(parse_optional_text(&row.father_name)),
        username: Set(parse_optional_text(&row.username)),
        control_id: Set(parse_optional_text(&row.control_id)),
        descrepency: Set(parse_optional_text(&row.descrepency)),
        university: Set(parse_optional_text(&row.university)),
        payment_mode: Set(parse_optional_text(&row.payment_mode)),
        trans_id: Set(parse_optional_text(&row.trans_id)),
        bank: Set(parse_optional_text(&row.bank)),
        rm: Set(parse_optional_text(&row.rm)),
        is_reconciled: Set(parse_flag(&row.is_reconciled)),
        online_exported: Set(parse_flag(&row.online_exported)),
    }
}

fn conflict() -> OnConflict {
    OnConflict::column(fee::Column::Id)
        .update_columns([
            fee::Column::AdmSession,
            fee::Column::AdmYear,
            fee::Column::Dod,
            fee::Column::Submit,
            fee::Column::Prog,
            fee::Column::Enroll,
            fee::Column::Student,
            fee::Column::YearSem,
            fee::Column::Category,
            fee::Column::Dob,
            fee::Column::Contact,
            fee::Column::Deposit,
            fee::Column::Nsd,
            fee::Column::Fee,
            fee::Column::Courses,
            fee::Column::Remarks,
            fee::Column::DepositBy,
            fee::Column::Ts,
            fee::Column::Medium,
            fee::Column::MotherName,
            fee::Column::FatherName,
            fee::Column::Username,
            fee::Column::ControlId,
            fee::Column::Descrepency,
            fee::Column::University,
            fee::Column::PaymentMode,
            fee::Column::TransId,
            fee::Column::Bank,
            fee::Column::Rm,
            fee::Column::IsReconciled,
            fee::Column::OnlineExported,
        ])
        .to_owned()
}

pub async fn upsert_rows<C: ConnectionTrait + TransactionTrait>(
    db: &C,
    rows: &[ParsedFeeRow],
) -> Result<SyncReport, String> {
    let ids: Vec<i32> = rows.iter().map(|r| r.id).collect();
    let mut existing: std::collections::HashSet<i32> = std::collections::HashSet::new();
    for chunk in ids.chunks(500) {
        let found = FeeEntity::find()
            .filter(fee::Column::Id.is_in(chunk.to_vec()))
            .all(db)
            .await
            .map_err(|e| e.to_string())?;
        existing.extend(found.into_iter().map(|r| r.id));
    }
    let inserted = rows.iter().filter(|r| !existing.contains(&r.id)).count() as u64;
    let updated = rows.iter().filter(|r| existing.contains(&r.id)).count() as u64;

    let txn = db.begin().await.map_err(|e| e.to_string())?;
    for chunk in rows.chunks(BATCH_SIZE) {
        let models: Vec<fee::ActiveModel> = chunk.iter().map(to_active).collect();
        if models.is_empty() {
            continue;
        }
        FeeEntity::insert_many(models)
            .on_conflict(conflict())
            .exec(&txn)
            .await
            .map_err(|e| e.to_string())?;
    }
    txn.commit().await.map_err(|e| e.to_string())?;
    Ok(SyncReport {
        inserted,
        updated,
        skipped: 0,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn excel_serial_matches_sample_dod() {
        let d = excel_serial_to_date(43806.0).expect("date");
        assert_eq!(d, NaiveDate::from_ymd_opt(2019, 12, 7).unwrap());
    }
}
