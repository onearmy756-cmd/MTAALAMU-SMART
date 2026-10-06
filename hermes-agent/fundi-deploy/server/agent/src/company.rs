//! company.rs — REMOTE ZA KAMPUNI NZIMA KUPITIA WIREGUARD (maelezo ya mmiliki):
//! "kuremote computer za kampuni husika kwa kutumia VPN wireguard" — kampuni
//! ina matawi (branches); kila tawi lina PCs zake; admin wa kampuni anasimamia
//! ZOTE kutoka dashboards moja kupitia wg0 (kila tawi = subnet/peers za VPN).
//!
//! HALISI: branches/PCs zinawekwa na admin (SQLite); health/per-PC stats zinakuja
//! kutoka remote_view::full_view (probe halisi) — hakuna uongo.

use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Branch {
    pub id: String,
    pub name: String,        // mf. "Tawi la Dar", "Tawi la Mwanza"
    pub city: String,
    pub vpn_subnet: String,  // subnet ya WireGuard ya tawi (mf. 10.66.66.0/24)
    pub created_at: String,
}

pub async fn init_tables(db: &SqlitePool) {
    let _ = sqlx::query(
        "CREATE TABLE IF NOT EXISTS branches (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            city TEXT NOT NULL,
            vpn_subnet TEXT NOT NULL,
            created_at TEXT NOT NULL
        )",
    )
    .execute(db)
    .await;
}

pub async fn add_branch(db: &SqlitePool, id: &str, name: &str, city: &str, vpn_subnet: &str) -> Result<Branch, String> {
    if id.trim().is_empty() || name.trim().is_empty() {
        return Err("id na name ni lazima".into());
    }
    // Subnet check rahisi (a.b.c.d/len)
    let ok = {
        let parts: Vec<&str> = vpn_subnet.split('/').collect();
        parts.len() == 2
            && parts[0].split('.').count() == 4
            && parts[1].parse::<u8>().map(|l| l <= 32).unwrap_or(false)
    };
    if !ok {
        return Err(format!("vpn_subnet '{vpn_subnet}' si sahihi (mf. 10.66.66.0/24)"));
    }
    let created = chrono::Local::now().to_rfc3339();
    sqlx::query("INSERT OR REPLACE INTO branches (id, name, city, vpn_subnet, created_at) VALUES (?,?,?,?,?)")
        .bind(id.trim())
        .bind(name.trim())
        .bind(city.trim())
        .bind(vpn_subnet.trim())
        .bind(&created)
        .execute(db)
        .await
        .map_err(|e| e.to_string())?;
    Ok(Branch {
        id: id.trim().into(),
        name: name.trim().into(),
        city: city.trim().into(),
        vpn_subnet: vpn_subnet.trim().into(),
        created_at: created,
    })
}

pub async fn list_branches(db: &SqlitePool) -> Vec<Branch> {
    let rows: Vec<(String, String, String, String, String)> = sqlx::query_as(
        "SELECT id, name, city, vpn_subnet, created_at FROM branches ORDER BY created_at",
    )
    .fetch_all(db)
    .await
    .unwrap_or_default();
    rows.into_iter()
        .map(|(id, name, city, vpn_subnet, created_at)| Branch { id, name, city, vpn_subnet, created_at })
        .collect()
}

/// Muhtasari wa kampuni: kila tawi + PCs wake (hosts za remote_view).
pub async fn company_summary(db: &SqlitePool, hosts_by_branch: &[(String, Vec<(String, String)>)], timeout_ms: u64) -> serde_json::Value {
    let branches = list_branches(db).await;
    let mut out = Vec::new();
    for b in &branches {
        let hosts: Vec<(String, String)> = hosts_by_branch
            .iter()
            .find(|(bid, _)| bid == &b.id)
            .map(|(_, h)| h.clone())
            .unwrap_or_default();
        let views = crate::remote_view::fleet_snapshot(db, &hosts, timeout_ms).await;
        let online = views.iter().filter(|v| v.online).count();
        let problems: u32 = views.iter().map(|v| v.problems.len() as u32).sum();
        out.push(serde_json::json!({
            "branch": { "id": b.id, "name": b.name, "city": b.city, "vpn_subnet": b.vpn_subnet },
            "pcs_total": views.len(),
            "pcs_online": online,
            "pcs_offline": views.len() - online,
            "problems_open": problems,
            "pcs": views,
        }));
    }
    serde_json::json!({
        "ok": true,
        "branches": out,
        "note_sw": "Kampuni nzima kupitia WireGuard — kila tawi ni subnet ya wg0; kila PC ina mwonekano kamili (probe halisi)."
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn matawi_yanaongezwa_na_kuhifadhiwa() {
        let db = sqlx::SqlitePool::connect("sqlite::memory:").await.unwrap();
        init_tables(&db).await;
        add_branch(&db, "dar", "Tawi la Dar", "Dar es Salaam", "10.66.66.0/24").await.unwrap();
        add_branch(&db, "mwa", "Tawi la Mwanza", "Mwanza", "10.66.67.0/24").await.unwrap();
        let bs = list_branches(&db).await;
        assert_eq!(bs.len(), 2);
        assert_eq!(bs[0].city, "Dar es Salaam");
        assert_eq!(bs[1].vpn_subnet, "10.66.67.0/24");
    }

    #[tokio::test]
    async fn subnet_mbaya_inakataliwa() {
        let db = sqlx::SqlitePool::connect("sqlite::memory:").await.unwrap();
        init_tables(&db).await;
        assert!(add_branch(&db, "x", "X", "X", "sio-subnet").await.is_err());
        assert!(add_branch(&db, "x", "X", "X", "10.66.66.0/99").await.is_err());
        assert!(add_branch(&db, "", "", "", "10.66.66.0/24").await.is_err());
    }

    #[tokio::test]
    async fn company_summary_inajumuisha_matawi_yote() {
        let db = sqlx::SqlitePool::connect("sqlite::memory:").await.unwrap();
        init_tables(&db).await;
        crate::daily::init_tables(&db).await; // problems tables
        add_branch(&db, "dar", "Dar", "Dar es Salaam", "10.66.66.0/24").await.unwrap();
        add_branch(&db, "mwa", "Mwanza", "Mwanza", "10.66.67.0/24").await.unwrap();
        let hosts = vec![
            ("dar".to_string(), vec![("hr".to_string(), "127.0.0.1".to_string())]),
            ("mwa".to_string(), vec![]),
        ];
        let s = company_summary(&db, &hosts, 20).await;
        assert_eq!(s["branches"].as_array().unwrap().len(), 2);
        assert_eq!(s["branches"][0]["pcs_total"], 1);
        assert_eq!(s["branches"][1]["pcs_total"], 0);
    }
}
