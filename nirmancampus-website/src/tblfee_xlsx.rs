//! Parse tblfee.xlsx and upsert rows by Receipt ID.

use std::collections::BTreeMap;
use std::io::Cursor;

use calamine::{Data, Reader, Xlsx, open_workbook_from_rs};
use chrono::{NaiveDate, TimeDelta, Utc};
use sea_orm::{
    ActiveValue::Set, ColumnTrait, ConnectionTrait, EntityTrait, QueryFilter, TransactionTrait,
    sea_query::OnConflict,
};

use crate::entities::tblfee::{self, Entity as TblfeeEntity};

const BATCH_SIZE: usize = 20;

#[derive(Clone, Debug, Default)]
pub struct ParsedFeeRow {
    pub id: i64,
    pub adm_session: String,
    pub adm_year: String,
    pub dod: Option<NaiveDate>,
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

fn cell_to_date(cell: &Data) -> Option<NaiveDate> {
    match cell {
        Data::Empty => None,
        Data::DateTime(dt) => excel_serial_to_date(dt.as_f64()),
        Data::Float(f) => excel_serial_to_date(*f),
        Data::Int(i) => excel_serial_to_date(*i as f64),
        Data::String(s) => parse_date_string(s),
        Data::DateTimeIso(s) => parse_date_string(s),
        _ => None,
    }
}

fn parse_date_string(s: &str) -> Option<NaiveDate> {
    let s = s.trim();
    const FMTS: &[&str] = &["%d-%m-%Y", "%d/%m/%Y", "%Y-%m-%d", "%d-%m-%Y %H:%M:%S"];
    for fmt in FMTS {
        if let Ok(d) = NaiveDate::parse_from_str(s, fmt) {
            return Some(d);
        }
        if let Ok(dt) = chrono::NaiveDateTime::parse_from_str(s, fmt) {
            return Some(dt.date());
        }
    }
    if let Ok(n) = s.parse::<f64>() {
        return excel_serial_to_date(n);
    }
    None
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

fn parse_id(row: &[Data], idx: Option<usize>) -> Option<i64> {
    let raw = cell_at(row, idx);
    let raw = raw.trim();
    if raw.is_empty() {
        return None;
    }
    if let Ok(n) = raw.parse::<i64>() {
        if n > 0 {
            return Some(n);
        }
        return None;
    }
    if let Ok(f) = raw.parse::<f64>() {
        let n = f.trunc() as i64;
        if n > 0 {
            return Some(n);
        }
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
    let mut by_id: BTreeMap<i64, ParsedFeeRow> = BTreeMap::new();
    for row in row_iter {
        if row.iter().all(|c| matches!(c, Data::Empty)) {
            continue;
        }
        let Some(id) = parse_id(row, Some(id_idx)) else {
            skipped += 1;
            continue;
        };
        let dod_val = dod.and_then(|i| row.get(i)).and_then(cell_to_date);
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

fn to_active(row: &ParsedFeeRow, now: chrono::DateTime<Utc>) -> tblfee::ActiveModel {
    tblfee::ActiveModel {
        id: Set(row.id),
        created_at: Set(Some(now)),
        updated_at: Set(Some(now)),
        adm_session: Set(row.adm_session.clone()),
        adm_year: Set(row.adm_year.clone()),
        dod: Set(row.dod),
        submit: Set(row.submit.clone()),
        prog: Set(row.prog.clone()),
        enroll: Set(row.enroll.clone()),
        student: Set(row.student.clone()),
        year_sem: Set(row.year_sem.clone()),
        category: Set(row.category.clone()),
        dob: Set(row.dob.clone()),
        contact: Set(row.contact.clone()),
        deposit: Set(row.deposit.clone()),
        nsd: Set(row.nsd.clone()),
        fee: Set(row.fee.clone()),
        courses: Set(row.courses.clone()),
        remarks: Set(row.remarks.clone()),
        deposit_by: Set(row.deposit_by.clone()),
        ts: Set(row.ts.clone()),
        medium: Set(row.medium.clone()),
        mother_name: Set(row.mother_name.clone()),
        father_name: Set(row.father_name.clone()),
        username: Set(row.username.clone()),
        control_id: Set(row.control_id.clone()),
        descrepency: Set(row.descrepency.clone()),
        university: Set(row.university.clone()),
        payment_mode: Set(row.payment_mode.clone()),
        trans_id: Set(row.trans_id.clone()),
        bank: Set(row.bank.clone()),
        rm: Set(row.rm.clone()),
        is_reconciled: Set(row.is_reconciled.clone()),
        online_exported: Set(row.online_exported.clone()),
    }
}

fn conflict() -> OnConflict {
    OnConflict::column(tblfee::Column::Id)
        .update_columns([
            tblfee::Column::UpdatedAt,
            tblfee::Column::AdmSession,
            tblfee::Column::AdmYear,
            tblfee::Column::Dod,
            tblfee::Column::Submit,
            tblfee::Column::Prog,
            tblfee::Column::Enroll,
            tblfee::Column::Student,
            tblfee::Column::YearSem,
            tblfee::Column::Category,
            tblfee::Column::Dob,
            tblfee::Column::Contact,
            tblfee::Column::Deposit,
            tblfee::Column::Nsd,
            tblfee::Column::Fee,
            tblfee::Column::Courses,
            tblfee::Column::Remarks,
            tblfee::Column::DepositBy,
            tblfee::Column::Ts,
            tblfee::Column::Medium,
            tblfee::Column::MotherName,
            tblfee::Column::FatherName,
            tblfee::Column::Username,
            tblfee::Column::ControlId,
            tblfee::Column::Descrepency,
            tblfee::Column::University,
            tblfee::Column::PaymentMode,
            tblfee::Column::TransId,
            tblfee::Column::Bank,
            tblfee::Column::Rm,
            tblfee::Column::IsReconciled,
            tblfee::Column::OnlineExported,
        ])
        .to_owned()
}

pub async fn upsert_rows<C: ConnectionTrait + TransactionTrait>(
    db: &C,
    rows: &[ParsedFeeRow],
) -> Result<SyncReport, String> {
    let now = Utc::now();
    let ids: Vec<i64> = rows.iter().map(|r| r.id).collect();
    let mut existing: std::collections::HashSet<i64> = std::collections::HashSet::new();
    for chunk in ids.chunks(500) {
        let found = TblfeeEntity::find()
            .filter(tblfee::Column::Id.is_in(chunk.to_vec()))
            .all(db)
            .await
            .map_err(|e| e.to_string())?;
        existing.extend(found.into_iter().map(|r| r.id));
    }
    let inserted = rows.iter().filter(|r| !existing.contains(&r.id)).count() as u64;
    let updated = rows.iter().filter(|r| existing.contains(&r.id)).count() as u64;

    let txn = db.begin().await.map_err(|e| e.to_string())?;
    for chunk in rows.chunks(BATCH_SIZE) {
        let models: Vec<tblfee::ActiveModel> =
            chunk.iter().map(|row| to_active(row, now)).collect();
        if models.is_empty() {
            continue;
        }
        TblfeeEntity::insert_many(models)
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
