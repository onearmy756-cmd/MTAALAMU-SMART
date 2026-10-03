//! FUNDI MOBILE — library (agentic mobile repair, HITL daima)

pub mod agentic;
pub mod b2b;
pub mod brands;
pub mod consent;
pub mod datarec;
pub mod devices;
pub mod drivers;
pub mod emailagent;
pub mod errcodes;
pub mod govagent;
pub mod licenses;
pub mod logo;
pub mod mlinzi;
pub mod netagentic;
pub mod netcalc;
pub mod netdiag;
pub mod osremote;
pub mod payments;
pub mod pdf;
pub mod procedures;
pub mod proagentic;
pub mod proservices;
pub mod recover;
pub mod resetagent;
pub mod restore;
pub mod scribe;
pub mod services;
pub mod shield;
pub mod sysvision;

pub use agentic::{run as agentic_run, dashboard, list_jobs, AgenticRequest, Job};
pub use brands::{button_phone, flash_info, recovery_combo};
pub use consent::{record as record_consent, verify as verify_consent, Consent};
pub use procedures::{android_reset_adb, android_reset_recovery, button_reset, diagnostics, flash_firmware};
pub use recover::{list as recover_list, preflight as recover_preflight, google_remote, samsung_remote, xiaomi_remote, huawei_remote, owner_adb, recovery_reset, frp_aftermath, backup_first as recover_backup, ownership as recover_ownership, report as recover_report, SERVICE_ID as RECOVERY_SERVICE_ID};
pub use services::get as get_service;
pub use proservices::{catalog_sw as pro_catalog, show_sw as pro_show, price as pro_price, load as pro_load};
pub use proagentic::{run as pro_run, list_jobs as pro_jobs, dashboard as pro_dashboard, book_html as pro_book, ProRequest};
pub use netcalc::{calc as net_calc, list as net_calc_list, formula_sw as net_formula, problems_sw as net_problems};
pub use netdiag::diagnose_blocking as net_diagnose;
pub use netagentic::{run as net_run, list_jobs as net_jobs, book_html as net_book, summary as net_summary, NetRequest};
pub use sysvision::scan as sys_scan;
pub use shield::{audit as shield_audit, scan as shield_scan, clean as shield_clean, report as shield_report, guard_baseline, guard_check, guard_install, guard_status};
pub use errcodes::{list as errcodes_list, solve as errcodes_solve};
pub use resetagent::{run as reset_run, list_jobs as reset_jobs, book_html as reset_book, ResetRequest};
pub use restore::{verify as restore_verify, plan as restore_plan, execute as restore_execute, list as restore_list};
pub use osremote::{request as osi_request, status as osi_status, bundles as osi_bundles, watch as osi_watch, print_status as osi_print};
pub use govagent::{systems as gov_systems, intake_interactive as gov_intake, fill_and_print as gov_fill, speak_sw as gov_speak, searxng_search as gov_search, jamii_list as jamii_list, jamii_show as jamii_show, letter as gov_letter};
pub use emailagent::{list as email_list, check as email_check, setup as email_setup, password as email_password, problems as email_problems, providers_json as email_providers_json};
pub use licenses::{status as license_status, office_status as license_office, activate as license_activate, activate_office as license_activate_office, genuine as license_genuine};
pub use drivers::{list as drivers_list, scan as drivers_scan, update as drivers_update};
pub use mlinzi::{forward_check as guard_forward, forward_check_adb as guard_forward_adb, forward_stop as guard_forward_stop, sms as guard_sms, link as guard_link, email as guard_email, academy as guard_academy, status as guard_status_all, summary_json as guard_summary_json};
pub use datarec::{scan as data_scan, plan as data_plan, recycle as data_recycle, photorec as data_photorec, phone as data_phone};
pub use scribe::{start as scribe_start, live as scribe_live, listen as scribe_listen, finish as scribe_finish, sessions as scribe_sessions, add as scribe_add};
pub use payments::{invoice_add, checkout, confirm as confirm_payment, report as revenue_report, invoice_html};
pub use b2b::{
    doc_html as b2b_html, doc_pdf as b2b_pdf, quote_add, quote_to_invoice, report as b2b_report,
    set_status as b2b_status,
};
pub use logo::{funi_logo_png, LOGO_SVG};
pub use pdf::{b2b_doc_pdf, deploy_report_pdf};
