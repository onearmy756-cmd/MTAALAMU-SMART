const fs = require('fs');
const path = require('path');

const trades = [
  { id: 'computer', label_sw: 'Kompyuta', label_en: 'Computer' },
  { id: 'simu', label_sw: 'Simu', label_en: 'Phone' },
  { id: 'umeme', label_sw: 'Umeme', label_en: 'Electrical' },
  { id: 'maji', label_sw: 'Maji', label_en: 'Water/Plumbing' },
  { id: 'gari', label_sw: 'Gari', label_en: 'Car/Mechanic' },
  { id: 'hvac', label_sw: 'AC/Jokofu', label_en: 'HVAC/AC' },
  { id: 'cctv', label_sw: 'CCTV', label_en: 'CCTV' },
  { id: 'solar', label_sw: 'Solar', label_en: 'Solar Power' },
  { id: 'useremala', label_sw: 'Useremala', label_en: 'Upholstery' },
  { id: 'ujenzi', label_sw: 'Ujenzi', label_en: 'Construction' },
  { id: 'rangi', label_sw: 'Rangi', label_en: 'Painting' },
  { id: 'welding', label_sw: 'Welding', label_en: 'Welding' },
  { id: 'pikipiki', label_sw: 'Pikipiki', label_en: 'Motorcycle' },
  { id: 'jenereta', label_sw: 'Jenereta', label_en: 'Generator' },
  { id: 'gas', label_sw: 'Gesi', label_en: 'Gas Appliance' },
  { id: 'roofing', label_sw: 'Roofing', label_en: 'Roofing' },
  { id: 'pump', label_sw: 'Pampu', label_en: 'Water Pump' },
  { id: 'tailor', label_sw: 'Tailor', label_en: 'Tailoring' },
  { id: 'appliance', label_sw: 'Vifaa vya Nyumba', label_en: 'Home Appliances' },
  { id: 'borehole', label_sw: 'Borehole', label_en: 'Borehole' },
  { id: 'gate_motor', label_sw: 'Gate Motor', label_en: 'Gate Motor' }
];

const tradeConfigs = {
  computer: {
    symptoms: ['no_boot','slow_pc','blue_screen','overheat','virus_popup','no_network','no_display','usb_fail','fan_noise','hard_drive_sound','ram_error','keyboard_fail','mouse_fail','cd_dvd_fail','wifi_drop','bluetooth_fail','speaker_no_sound','mic_fail','bsod_loop','bios_error'],
    causes: ['psu_fail','ram_bad','hdd_fail','virus','software_corrupt','overheat','motherboard_fault','gpu_fault','loose_cable','dust_build']
  },
  simu: {
    symptoms: ['no_power','screen_crack','battery_drain','no_charge','boot_loop','slow_ui','no_signal','wifi_fail','bluetooth_fail','camera_fail','speaker_fail','mic_fail','sensor_fail','water_damage','overheat','no_network','sim_error','password_lock','frp_lock','button_fail'],
    causes: ['battery_dead','screen_broken','charging_port_bad','motherboard_fault','software_corrupt','water_damage','sim_card_fail','antenna_broken','camera_module_fail','malware_virus']
  },
  umeme: {
    symptoms: ['breaker_trip','sparks','shock','no_power','flicker','buzzing_sound','socket_burn','bulb_blow','earthing_fail','voltage_high','voltage_low','rcd_trip','wire_loose','switch_fail','distribution_fail','transformer_fail','meter_error','cable_burn','power_surge','ground_fault'],
    causes: ['overload','short_circuit','earth_leak','loose_wire','bad_breaker','bad_switch','faulty_socket','cable_damage','voltage_fluctuation','grounding_bad']
  },
  maji: {
    symptoms: ['pipe_leak','low_pressure','no_water','clogged_drain','toilet_run','water_heater_fail','faucet_drip','noise_pipe','sewer_backup','water_taste_bad','water_color_brown','rusty_pipe','valve_leak','pump_fail','water_meter_stop','shower_low_pressure','sink_slow_drain','wc_blocked','hose_burst','roof_leak_plumb'],
    causes: ['pipe_damage','clogged_pipe','valve_fault','pump_fail','pressure_reg_fail','water_heater_bad','sewer_block','tap_washer_worn','tank_float_fail','pipe_corrosion']
  },
  gari: {
    symptoms: ['no_start','black_smoke','white_smoke','blue_smoke','overheating','metal_noise','check_engine','hard_start','stalling','poor_fuel_econ','gear_shift_hard','brake_fail','steering_shake','suspension_noise','ac_not_cold','battery_drain','alternator_fail','starter_fail','tyre_wear','oil_leak'],
    causes: ['battery_bad','starter_fail','fuel_pump_fail','injector_bad','air_filter_clog','head_gasket_burst','piston_rings_worn','radiator_block','thermostat_fail','brake_pad_worn']
  },
  hvac: {
    symptoms: ['no_cool','no_heat','ac_no_start','loud_noise','water_leak','remote_fail','error_code','frozen_coil','weak_airflow','bad_smell_ac','high_electric','compressor_noise','fan_not_spin','thermostat_bad','duct_leak','refrigerant_low','ac_short_cycle','indoor_noise','outdoor_vibrate','filter_clog'],
    causes: ['thermostat_fail','compressor_dead','start_capacitor_bad','coil_dirty','refrigerant_leak','filter_clog','fan_motor_fail','duct_leak','control_board_bad','evaporator_frozen']
  },
  cctv: {
    symptoms: ['no_video','black_screen','blurry_image','no_recording','camera_offline','remote_view_fail','motion_detect_fail','night_vision_bad','distorted_image','cable_fail','nvr_no_start','hdd_full','hdd_fail','ip_conflict','camera_burn','poe_fail','audio_bad','wifi_cam_drop','ptz_fail','storage_error'],
    causes: ['power_supply_bad','cable_broken','hdd_fail','nvr_fault','camera_sensor_bad','network_conflict','poe_switch_fail','firmware_outdated','connector_bad','lens_dirty']
  },
  solar: {
    symptoms: ['no_power','battery_not_charge','inverter_fail','low_voltage','panel_hot','solar_not_charge','error_code_solar','battery_swell','load_not_run','inverter_alarm','grid_fail_solar','mppt_fail','panel_crack','cable_solar_burn','low_output','battery_leak','disconnect_switch','controller_fail','inverter_fan_fail','monitoring_offline'],
    causes: ['panel_fault','battery_bad','inverter_fail','mppt_fail','wiring_loose','charge_ctrl_bad','shading_issue','bms_fault','grid_voltage_bad','fuse_blown']
  },
  useremala: {
    symptoms: ['sofa_sagging','spring_noise','frame_broken','fabric_tear','cushion_flat','armrest_broken','leg_broken','stain_damage','seat_sink','recliner_fail','bed_squeak','drawer_stuck','carpet_tear','curtain_fall','ottoman_broken','webbing_break','button_pop','foam_degrade','staple_out','joint_loose'],
    causes: ['spring_broken','foam_worn','frame_crack','webbing_stretch','fabric_damaged','joint_loose','staple_fail','cushion_worn','leg_damaged','mechanism_broken']
  },
  ujenzi: {
    symptoms: ['wall_crack','floor_uneven','ceiling_leak','door_jam','window_jam','concrete_crumble','plaster_fall','tile_crack','beam_deflect','roof_sag','foundation_crack','damp_wall','mold_wall','paint_peel_uj','stair_crack','sewer_slope_bad','rebar_rust','scaffold_loose','block_shift','mortar_weak'],
    causes: ['foundation_settle','moisture_damage','poor_workmanship','material_bad','water_seepage','structural_overload','steel_corrosion','design_flaw','soil_movement','weather_damage']
  },
  rangi: {
    symptoms: ['peeling_paint','cracked_paint','bubbling_paint','faded_paint','stain_mark','brush_mark','roller_mark','uneven_coat','color_variation','mildew_paint','chalking','alligatoring','sagging_paint','wrinkling','graffiti','smoke_stain','water_stain_r','rust_bleed','poor_adhesion','lap_mark'],
    causes: ['poor_surface_prep','moisture','paint_old','wrong_primer','extreme_temp','thick_coat','low_quality_paint','dirty_surface','alkali_attack','uv_damage']
  },
  welding: {
    symptoms: ['weld_crack','porosity_weld','spatter_excess','undercut','incomplete_penetration','cold_lap','slag_inclusion','distortion_weld','burn_through','weld_weak','weld_rust','electrode_stick','arc_irregular','weld_dirty','bead_irregular','gas_hole_w','warp_panel','tungsten_contam','shield_gas_fail','post_weld_crack'],
    causes: ['wrong_current','dirty_base_metal','wrong_gas','electrode_wrong','bad_technique','speed_too_fast','base_metal_thin','insufficient_heat','joint_prep_bad','contamination']
  },
  pikipiki: {
    symptoms: ['no_start_p','engine_noise_p','brake_weak_p','chain_slip','overheat_p','tyre_flat','smoke_p','fuel_leak_p','gear_shift_fail','headlight_bad','indicator_fail','battery_p_fail','carburetor_bad','clutch_slip','fork_shake','wheel_bearing_p','oil_leak_p','speedo_fail','kick_start_fail','throttle_stick'],
    causes: ['spark_plug_bad','carburetor_clog','fuel_tap_bad','battery_p_bad','brake_pad_worn','chain_stretch','air_filter_clog_p','oil_low','clutch_worn','tyre_worn_p']
  },
  jenereta: {
    symptoms: ['no_start_g','low_voltage','no_voltage','smoke_g','overheat_g','high_fuel','engine_knock','oil_pressure_low','alternator_fail_g','avr_fail','voltage_fluctuate','starter_g_fail','battery_g_fail','fuel_leak_g','oil_leak_g','exhaust_noise','vibration_g','surge_g','governor_fail','coolant_g_leak'],
    causes: ['fuel_bad','battery_g_bad','spark_plug_bad_g','avr_bad','alternator_bad','fuel_filter_clog','oil_level_low','carburetor_bad_g','starter_bad_g','coolant_g_low']
  },
  gas: {
    symptoms: ['gas_leak_smell','pilot_goes_out','no_flame','yellow_flame','low_flame','burner_clog_g','oven_not_heat','geyser_fail','gas_smell_strong','flame_backfire','burner_rust','oven_thermostat_fail','gas_valve_bad','igniter_fail','flame_lift','cooktop_burner_bad','gas_pipe_bend','regulator_fail','gas_meter_stop','hose_crack_g'],
    causes: ['gas_leak','pilot_dirty','burner_clog','thermocouple_fail','gas_valve_bad','regulator_bad','igniter_fail','gas_pressure_bad','hose_crack','vent_block']
  },
  roofing: {
    symptoms: ['roof_leak','missing_shingle','crack_sheet','rust_sheet','nail_pops','flashing_leak','gutter_clog','gutter_leak','ridge_cap_bad','valley_leak','skylight_leak','sag_roof','wind_damage_rf','hail_damage','granule_loss','eave_rot','fascia_rot','soffit_vent_clog','chimney_leak','drip_edge_bad'],
    causes: ['sheet_damaged','flashing_fail','nail_bad','sealant_old','gutter_clog_r','wood_rot','wind_damage','hail_damage','ponding_water','fastener_backout']
  },
  pump: {
    symptoms: ['no_water_p','low_flow_p','no_pressure','pump_rattle','pump_short_cycle','no_prime','pump_overheat','motor_smell_p','pressure_tank_fail','leak_pump_seal','impeller_bad','check_valve_bad','well_drop_in_water','motor_draw_high_amp','pump_vibrate','strainer_clog_p','pipe_suction_leak','pressure_switch_bad','control_box_fail','capacitor_bad_p'],
    causes: ['impeller_clog','motor_burn','seal_leak','pressure_switch_bad','capacitor_fail','check_valve_bad','suction_leak','strainer_clog','well_yield_low','tank_bladder_fail']
  },
  tailor: {
    symptoms: ['zip_broken','seam_rip','button_missing','hem_fall','size_too_big','size_too_small','fabric_tear_t','lining_loose','pocket_broken','stitch_skip','thread_break','needle_break','elastic_worn','hook_eye_bad','buckle_broken','pleat_lose','cuff_tear','collar_fray','waistband_loose','patch_needed'],
    causes: ['seam_weak','zip_damaged','fabric_worn','stitch_bad','lining_fail','buttonhole_bad','elastic_dead','thread_bad','needle_dull','pattern_error']
  },
  appliance: {
    symptoms: ['washer_no_spin','fridge_no_cool','microwave_no_heat','tv_no_pic','iron_no_heat','kettle_no_boil','cooker_no_heat','dryer_no_heat','dishwasher_no_water','freezer_frost_build','blender_not_spin','vacuum_no_suck','coffee_no_brew','toaster_not_heat','fan_not_turn','heater_not_glow','rice_not_cook','juicer_no_juice','mixer_grind_fail','food_processor_bad'],
    causes: ['heating_element_bad','motor_burned','belt_broken_a','thermal_fuse_blow','thermostat_bad_a','control_board_bad_a','door_switch_bad','pump_clog_a','fan_motor_bad','power_cord_bad']
  },
  borehole: {
    symptoms: ['no_water_bh','low_yield','water_dirty','sand_in_water','pump_not_reach','low_pressure_bh','air_in_line','bore_cave_in','casing_leak','screen_clog','water_level_drop','electrical_fail_bh','yield_reduce','water_odour','water_stain_bh','well_yield_test_bad','grout_fail','bore_deep_insufficient','pump_seize','controller_fail_bh'],
    causes: ['aquifer_deplete','screen_clog_bh','pump_bad_bh','casing_damage','sand_influx','water_table_drop','bore_collapse','seal_fail','motor_burn_bh','column_pipe_leak']
  },
  gate_motor: {
    symptoms: ['gate_no_open','gate_no_close','gate_slow','motor_noise_gm','remote_no_work','obstacle_sensor_fail','gate_half_stop','gate_shake','chain_slip_gm','rack_teeth_wear','limit_switch_bad','control_board_bad_gm','power_supply_bad_gm','battery_backup_fail','safety_beam_fail','manual_release_fail','gear_strip','motor_smell_gm','gate_jerky','indicator_led_blink'],
    causes: ['motor_burn_gm','control_board_fault','remote_battery_dead','limit_switch_mis','chain_drive_bad','safety_beam_mis','power_surge_gm','capacitor_bad_gm','rack_pinion_wear','obstacle_sensor_bad']
  }
};

const problems = [];
let idCounter = 1;

trades.forEach(trade => {
  const cfg = tradeConfigs[trade.id];
  for (let i = 0; i < 20; i++) {
    const pid = 'prob_' + String(idCounter).padStart(4, '0');
    idCounter++;
    const sym1 = cfg.symptoms[i % cfg.symptoms.length];
    const sym2 = cfg.symptoms[(i + 3) % cfg.symptoms.length];
    const sym3 = cfg.symptoms[(i + 7) % cfg.symptoms.length];
    const cause1 = cfg.causes[i % cfg.causes.length];
    const cause2 = cfg.causes[(i + 2) % cfg.causes.length];
    const cause3 = cfg.causes[(i + 4) % cfg.causes.length];
    const p1 = +(0.35 + (i % 5) * 0.05).toFixed(2);
    const p2 = +(0.30 + ((i + 1) % 4) * 0.05).toFixed(2);
    const p3 = +(1.0 - p1 - p2).toFixed(2);
    const causesObj = {};
    causesObj[cause1] = p1;
    causesObj[cause2] = p2;
    causesObj[cause3] = p3;
    const sev = ((i % 5) + 1);
    const costBase = [10,15,20,25,30,35,40,45,50,55,60,70,80,90,100,120,140,160,180,200][i];
    const cost = costBase * 1000;
    const tm = [15,20,25,30,35,40,45,60,75,90,105,120,135,150,165,180,210,240,270,300][i];
    const sr = +(0.60 + (i % 8) * 0.05 + ((i % 3) * 0.01)).toFixed(2);
    const cs = [50,60,75,80,90,100,110,120,130,140,150,160,180,200,220,240,260,280,300,320][i];
    const formulaId = Math.random() > 0.4 ? null : 'formula_' + (100 + (i * 7) % 185);
    const descSw = `Tatizo la ${trade.label_sw} - ${sym1.replace(/_/g,' ')} na ${sym2.replace(/_/g,' ')} (kesi ${i+1}/20)`;
    const descEn = `${trade.label_en} Problem - ${sym1.replace(/_/g,' ')} and ${sym2.replace(/_/g,' ')} (case ${i+1}/20)`;
    const solSw = `Kutatua kwa ${cause1.replace(/_/g,' ')}: angalia vifaa na ukarabate kama ilivyoelekezwa. Pia kagua ${cause2.replace(/_/g,' ')} na ${cause3.replace(/_/g,' ')}.`;
    const solEn = `Resolve ${cause1.replace(/_/g,' ')}: inspect parts and repair as guided. Also check ${cause2.replace(/_/g,' ')} and ${cause3.replace(/_/g,' ')}.`;
    problems.push({
      id: pid,
      trade: trade.id,
      description: { sw: descSw, en: descEn },
      symptoms: [sym1, sym2, sym3],
      causes: causesObj,
      solution: { sw: solSw, en: solEn },
      formula: formulaId,
      time_min: tm,
      cost_tzs: cost,
      success_rate: sr,
      severity: sev,
      cases: cs
    });
  }
});

const out = { problems };
const outPath = path.join('c:', 'Users', 'STEPHANO AMAN', 'Desktop', 'MTAALAMU SMART', 'mtaalamu-smart', 'data', 'problems.json');
fs.writeFileSync(outPath, JSON.stringify(out, null, 2), 'utf8');
console.log('Wrote problems.json: ' + problems.length + ' problems');
console.log('Trades covered: ' + trades.length);
const counts = {};
problems.forEach(p => { counts[p.trade] = (counts[p.trade]||0)+1; });
console.log('Per trade:', JSON.stringify(counts));
