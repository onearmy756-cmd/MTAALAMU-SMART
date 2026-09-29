const fs = require('fs');
const path = require('path');

const OUT_DIR = path.join('c:', 'Users', 'STEPHANO AMAN', 'Desktop', 'MTAALAMU SMART', 'mtaalamu-smart', 'data', 'formulas_network');

const GROUPS = [
  { num: 1, code: 'KUNDI 1', id: 'signal', name_sw: 'Ishara na Nishati', name_en: 'Signal & Power', count: 15,
    topics: ['db_power','dbm_to_watts','voltage_gain_db','noise_voltage','SNR_calc','noise_floor_calc','thermal_noise','intermod_distortion','third_order_intercept','one_db_compression','dynamic_range_calc','phase_noise_hz','adjacent_channel_power','spurious_free_dynamic','crest_factor'] },
  { num: 2, code: 'KUNDI 2', id: 'antenna', name_sw: 'Antenna', name_en: 'Antenna', count: 20,
    topics: ['dipole_length_metre','yagi_gain_dbi','parabolic_gain','effective_aperture','radiation_resistance','beamwidth_degrees','front_to_back_ratio','return_loss_swr','polarization_loss','far_field_distance','path_loss_friis','link_margin','radar_range','eirp_calc','effective_radiated_power','effective_isotropic','gain_of_array','diversity_gain_combine','diversity_gain_maximum','effective_area_m2'] },
  { num: 3, code: 'KUNDI 3', id: 'propagation', name_sw: 'Uenezi wa Mawasiliano', name_en: 'Propagation', count: 15,
    topics: ['free_space_path_loss','two_ray_ground_reflection','okumura_hata','cost_231_hata','walfisch_ikegami','longley_rice','knife_edge_diffraction','rain_fade_margin','vegetation_loss','building_penetration','foliage_loss','urban_macro_loss','suburban_path_loss','rural_area_loss','indoor_path_loss'] },
  { num: 4, code: 'KUNDI 4', id: 'link_budget', name_sw: 'Mipango ya Link', name_en: 'Link Budget', count: 15,
    topics: ['downlink_budget','uplink_budget','eirp_transmit','isotropic_receive','path_loss_total','fade_margin_total','system_gain','c_no_ratio','eb_no_ratio','carrier_to_noise','available_margin','rain_fade_margin_lb','gaseous_absorption','antenna_feed_loss','cable_loss_total'] },
  { num: 5, code: 'KUNDI 5', id: 'modulation', name_sw: 'Modulation', name_en: 'Modulation', count: 15,
    topics: ['bpsk_ber','qpsk_ber','qam16_ber','qam64_ber','snr_vs_ber','spectral_efficiency','shannon_capacity','nyquist_rate','symbol_rate_calc','bit_rate_symbol','constellation_points','modulation_error_rate','error_vector_magnitude','phase_jitter_calc','adjacent_channel_leakage'] },
  { num: 6, code: 'KUNDI 6', id: 'traffic', name_sw: 'Mpiga Simu (Traffic)', name_en: 'Traffic Engineering', count: 15,
    topics: ['erlang_b_formula','erlang_c_wait','busy_hour_traffic','gos_blocking','grade_of_service','number_of_trunks','traffic_per_subscriber','bhca_calculation','average_hold_time','congestion_probability','queue_length_erlang','delay_probability','offered_traffic','carried_traffic','erlang_benchmark'] },
  { num: 7, code: 'KUNDI 7', id: 'ran', name_sw: 'RAN (Radio Access)', name_en: 'Radio Access Network', count: 15,
    topics: ['cell_range_maximum','path_loss_cell','link_budget_cell','site_sectorization','number_sites_area','frequency_reuse','interference_margin','soft_handover_gain','pilot_pollution','load_factor_calc','serving_sinr','hsdpa_throughput','lte_prb_utilization','5g_nr_throughput','carrier_aggregation_bw'] },
  { num: 8, code: 'KUNDI 8', id: 'fiber', name_sw: 'Fiber Optic', name_en: 'Fiber Optics', count: 20,
    topics: ['fiber_attenuation_db','chromatic_dispersion','pmd_dispersion','link_budget_fiber','splice_loss_total','connector_loss','fiber_range_max','db_km_cable','dispersion_limit','power_budget_fiber','rise_time_budget','eye_diagram_jitter','ber_vs_q_factor','osnr_calculation','edfa_gain_calc','fiber_kappa_nonlinear','rz_nrz_bandwidth','wdm_channel_spacing','number_wdm_channels','fiber_cutoff_wavelength'] },
  { num: 9, code: 'KUNDI 9', id: 'satellite', name_sw: 'Satellite', name_en: 'Satellite Comms', count: 15,
    topics: ['satellite_elevation_angle','geostationary_longitude','slant_range_geo','round_trip_delay','free_space_loss_sat','eirp_satellite','g_t_ratio','c_no_satellite','rain_fade_sat','antenna_diameter_gain','beamwidth_satellite','bandwidth_sat','capacity_shannon_sat','twta_amplifier','link_margin_sat'] },
  { num: 10, code: 'KUNDI 10', id: 'nr5g', name_sw: '5G NR', name_en: '5G New Radio', count: 15,
    topics: ['nr_arfcn_calc','ssb_frequency','prach_conf','numerology_scs','bandwidth_part_bw','prb_count_bw','max_throughput_5g','slot_duration','cp_length_calc','beam_sweep_gain','massive_mimo_gain','mu_mimo_pairing','spectral_eff_5g','latency_5g_urllc','coreset_resource'] },
  { num: 11, code: 'KUNDI 11', id: 'subnetting', name_sw: 'Subnetting IP', name_en: 'IP Subnetting', count: 20,
    topics: ['ipv4_subnet_mask','wildcard_mask','network_address','broadcast_address','hosts_per_subnet','subnet_count_cidr','supernet_cidr_summary','vlsm_subnet','ip_class_check','cidr_to_mask','mask_to_cidr','ipv6_prefix_length','number_ipv6_hosts','subnet_id_calc','gateway_address','dns_reverse_zone','ip_range_first_last','subnet_increment','binary_to_ipv4','ipv4_to_binary'] },
  { num: 12, code: 'KUNDI 12', id: 'bandwidth', name_sw: 'Uwezo wa Bandwidth', name_en: 'Bandwidth Planning', count: 15,
    topics: ['bandwidth_per_user','total_bandwidth_needed','throughput_vs_bandwidth','over_subscription_ratio','qos_reserved_bw','peak_vs_avg_bw','bandwidth_utilization_pct','bps_to_kbps','data_usage_monthly','voip_codec_bw','video_conference_bw','streaming_bitrate','dedicated_vs_shared','mpls_bw_class','burst_tolerance_calc'] },
  { num: 13, code: 'KUNDI 13', id: 'latency', name_sw: 'Latency (Kuchelewa)', name_en: 'Latency', count: 10,
    topics: ['one_way_latency','round_trip_time','propagation_delay_meters','processing_delay','queueing_delay_mm1','serialization_delay','jitter_calculation','packet_delay_variation','buffer_bloat_latency','total_latency_hops'] },
  { num: 14, code: 'KUNDI 14', id: 'routing', name_sw: 'Routing', name_en: 'Routing', count: 15,
    topics: ['rip_metric_hop','eigrp_metric_formula','ospf_cost_bw','bgp_path_selection','static_route_distance','administrative_distance','route_metric_compare','ospf_area_calc','vlan_id_range','mtu_path_discovery','route_summary_cidr','mpls_label_stack','as_path_prepend','policy_route_weight','ecmp_load_balance'] },
  { num: 15, code: 'KUNDI 15', id: 'qos', name_sw: 'Quality of Service', name_en: 'QoS', count: 10,
    topics: ['dscp_to_8021p','classification_priority','wfq_weight_allocation','token_bucket_rate','policing_burst_calc','shaping_rate_bw','queue_depth_drop','wred_probability','congestion_window_tcp','sla_guarantee_bw'] },
  { num: 16, code: 'KUNDI 16', id: 'wireless', name_sw: 'Wireless (WiFi)', name_en: 'Wireless (WiFi)', count: 15,
    topics: ['wifi_channel_overlap','2ghz_coverage_area','5ghz_range_calc','wifi_data_rate_80211ac','wifi_data_rate_80211ax','ofdm_subcarriers','mimo_spatial_stream','channel_bonding_bw','beacon_interval_effect','rssi_throughput_map','snr_required_modulation','wifi_capacity_per_ap','roaming_threshold_rssi','airtime_fairness','co_channel_interference'] },
  { num: 17, code: 'KUNDI 17', id: 'security', name_sw: 'Usalama (Security)', name_en: 'Security', count: 10,
    topics: ['password_entropy_bits','bcrypt_cost_factor','aes_key_length_sec','rsa_key_strength','hash_collision_bits','ssl_handshake_latency','firewall_rule_complexity','vpn_throughput_overhead','ids_signature_perf','zero_trust_score'] },
  { num: 18, code: 'KUNDI 18', id: 'monitoring', name_sw: 'Ufuatiliaji (Monitoring)', name_en: 'Monitoring', count: 10,
    topics: ['snmp_poll_interval','icmp_packet_loss','uptime_percentage','availability_nines','sla_mttr_calc','mean_between_fail','alert_threshold_pct','prometheus_metric_rate','log_volume_day','dashboard_refresh_rate'] },
  { num: 19, code: 'KUNDI 19', id: 'virtualization', name_sw: 'Virtualization', name_en: 'Virtualization', count: 10,
    topics: ['vm_sizing_ram_cpu','vm_consolidation_ratio','cpu_ready_pct','memory_overhead_vm','hyper_threading_ratio','vmdk_thin_thick','storage_iops_vm','network_vm_bandwidth','snapshot_size_calc','vm_migration_time'] },
  { num: 20, code: 'KUNDI 20', id: 'cloud', name_sw: 'Cloud Computing', name_en: 'Cloud Computing', count: 10,
    topics: ['ec2_instance_cost_mo','s3_storage_cost_gb','cloud_egress_cost','reserved_instance_saving','auto_scale_min_max','serverless_lambda_cost','cloudfront_bandwidth_cost','rds_storage_cost','multi_az_availability','tco_onprem_vs_cloud'] }
];

function pad2(n){return n<10?'0'+n:''+n;}
function fn(gid, i){return gid+'_'+pad2(i+1);}

function genInputs(group, idx){
  const base = [
    { name:'P_in', label:{sw:'Nishati In',en:'Input Power'}, unit:'W', default:1, min:0.001, max:1e6 },
    { name:'P_out', label:{sw:'Nishati Out',en:'Output Power'}, unit:'W', default:10, min:0.001, max:1e7 },
    { name:'V_in', label:{sw:'Voltage In',en:'Input Voltage'}, unit:'V', default:1, min:1e-6, max:10000 },
    { name:'V_out', label:{sw:'Voltage Out',en:'Output Voltage'}, unit:'V', default:3.16, min:1e-6, max:10000 },
    { name:'BW', label:{sw:'Bandwidth',en:'Bandwidth'}, unit:'Hz', default:1e6, min:1, max:1e12 },
    { name:'T', label:{sw:'Joto (K)',en:'Temperature'}, unit:'K', default:290, min:0, max:1000 },
    { name:'NF', label:{sw:'Noise Figure',en:'Noise Figure'}, unit:'dB', default:3, min:0, max:30 },
    { name:'SNR', label:{sw:'SNR',en:'SNR'}, unit:'dB', default:20, min:-20, max:100 }
  ];
  const extras = {
    antenna: [
      { name:'f_MHz', label:{sw:'Frequency (MHz)',en:'Frequency (MHz)'}, unit:'MHz', default:900, min:1, max:1e6 },
      { name:'D_m', label:{sw:'Diameter (m)',en:'Diameter (m)'}, unit:'m', default:2, min:0.01, max:100 },
      { name:'eta', label:{sw:'Ufanisi',en:'Efficiency'}, unit:'', default:0.6, min:0.01, max:1 },
      { name:'N', label:{sw:'Elements',en:'Elements'}, unit:'', default:4, min:1, max:256 }
    ],
    propagation: [
      { name:'f_GHz', label:{sw:'Frequency (GHz)',en:'Frequency (GHz)'}, unit:'GHz', default:2.1, min:0.01, max:100 },
      { name:'d_km', label:{sw:'Distance (km)',en:'Distance (km)'}, unit:'km', default:5, min:0.001, max:1000 },
      { name:'h_tx', label:{sw:'Tx Height (m)',en:'Tx Height'}, unit:'m', default:50, min:1, max:500 },
      { name:'h_rx', label:{sw:'Rx Height (m)',en:'Rx Height'}, unit:'m', default:5, min:1, max:300 }
    ],
    subnetting: [
      { name:'ipv4', label:{sw:'Anwani IPv4',en:'IPv4 Address'}, unit:'', default:'192.168.1.100', select:[{label:'192.168.1.100',value:'192.168.1.100'},{label:'10.0.0.5',value:'10.0.0.5'}] },
      { name:'cidr', label:{sw:'CIDR /prefix',en:'CIDR Prefix'}, unit:'', default:24, min:0, max:32 },
      { name:'mask', label:{sw:'Subnet Mask',en:'Subnet Mask'}, unit:'', default:'255.255.255.0' }
    ],
    nr5g: [
      { name:'freq_mhz', label:{sw:'Freq (MHz)',en:'Freq (MHz)'}, unit:'MHz', default:3500, min:400, max:52600 },
      { name:'scs', label:{sw:'Numerology SCS',en:'SCS'}, unit:'kHz', default:30, select:[{label:'15 kHz',value:15},{label:'30 kHz',value:30},{label:'60 kHz',value:60},{label:'120 kHz',value:120}] },
      { name:'bw_mhz', label:{sw:'Bandwidth',en:'Bandwidth'}, unit:'MHz', default:100, min:5, max:400 },
      { name:'layers', label:{sw:'MIMO Layers',en:'MIMO Layers'}, unit:'', default:4, min:1, max:16 }
    ],
    cloud: [
      { name:'vcpu', label:{sw:'vCPU',en:'vCPU'}, unit:'', default:4, min:0.25, max:256 },
      { name:'gb_ram', label:{sw:'RAM (GB)',en:'RAM (GB)'}, unit:'GB', default:16, min:0.5, max:4000 },
      { name:'hours', label:{sw:'Saa / Mwezi',en:'Hours / Month'}, unit:'h', default:730, min:1, max:8760 },
      { name:'gb_storage', label:{sw:'Storage (GB)',en:'Storage (GB)'}, unit:'GB', default:100, min:1, max:1e6 }
    ],
    fiber: [
      { name:'L_km', label:{sw:'Urefu (km)',en:'Length (km)'}, unit:'km', default:10, min:0.001, max:500 },
      { name:'alpha_db', label:{sw:'Attenuation dB/km',en:'Attenuation dB/km'}, unit:'dB/km', default:0.25, min:0.05, max:5 },
      { name:'D_ps', label:{sw:'Dispersion ps/nm/km',en:'Dispersion'}, unit:'ps/nm/km', default:17, min:0.1, max:200 },
      { name:'Tx_dBm', label:{sw:'Tx (dBm)',en:'Tx Power'}, unit:'dBm', default:0, min:-30, max:30 }
    ],
    traffic: [
      { name:'erl', label:{sw:'Erlang',en:'Erlang Traffic'}, unit:'E', default:10, min:0.1, max:1000 },
      { name:'lines', label:{sw:'Trunks',en:'Trunk Lines'}, unit:'', default:20, min:1, max:2000 },
      { name:'gos', label:{sw:'GoS',en:'Grade of Service'}, unit:'', default:0.02, min:0.001, max:0.2 },
      { name:'subs', label:{sw:'Watumiaji',en:'Subscribers'}, unit:'', default:1000, min:1, max:1e7 }
    ]
  };
  const arr = [];
  const pick = extras[group] ? [base[0], base[1], base[4], base[5], ...extras[group]] : base;
  const n = Math.min(5, pick.length);
  for (let k=0;k<n;k++){ arr.push(pick[(idx+k) % pick.length]); }
  return arr;
}

function genOutputs(group, topic, idx){
  const c = (e,unit,digits) => ({name:'out'+idx, label:{sw:`Matokeo ya ${topic.replace(/_/g,' ')}`,en:`Result: ${topic.replace(/_/g,' ')}`}, expr:e, unit:unit||'', digits:typeof digits==='number'?digits:3});
  const map = {
    db_power: [c('10 * log(P_out / P_in) / log(10)','dB',2)],
    dbm_to_watts: [c('pow(10, P_out / 10) / 1000','W',6)],
    voltage_gain_db: [c('20 * log(V_out / V_in) / log(10)','dB',2)],
    noise_voltage: [c('sqrt(4 * 1.3806e-23 * T * BW * pow(10, NF / 10))','V',8)],
    SNR_calc: [c('P_out / (4 * 1.3806e-23 * T * BW * pow(10, NF / 10))','',3)],
    default: [c('(P_out + P_in + V_out + V_in) / 4','',3)]
  };
  return map[topic] || map.default;
}

function genFormula(groupDef, fi){
  const gid = groupDef.id;
  const topic = groupDef.topics[fi % groupDef.topics.length];
  const id = fn(gid, fi);
  const sw = topic.replace(/_/g,' ').replace(/\b\w/g,l=>l.toUpperCase());
  const inputs = genInputs(gid, fi);
  const outputs = genOutputs(gid, topic, fi);
  const expr = outputs[0].expr;
  return {
    id,
    group: gid,
    name: { sw: sw, en: sw },
    formula: expr,
    description: { sw: `${groupDef.name_sw}: ${sw}`, en: `${groupDef.name_en}: ${sw}` },
    inputs,
    outputs,
    steps: [
      { label:'Formula', text: expr },
      { label:'Weka thamani', expr: expr, template:'Matokeo = {{result}}' }
    ],
    rules: [
      { check: `out${fi} >= 0`, status:'GOOD', msg:{sw:'Thamani nzuri',en:'Good value'} },
      { check: 'true', status:'WARNING', msg:{sw:'Angalia vigezo',en:'Check constraints'} }
    ],
    standards: { sw: [`Kundi ${groupDef.code}`], en: [`${groupDef.code}`] },
    test: {
      inputs: inputs.reduce((o,i)=>{ o[i.name]=i.default; return o; }, {}),
      expect: {}
    }
  };
}

GROUPS.forEach(g => {
  const formulas = [];
  for (let i=0; i<g.count; i++){ formulas.push(genFormula(g, i)); }
  const file = {
    group: {
      id: g.id,
      code: g.code,
      label: { sw: `Kundi ${pad2(g.num)}: ${g.name_sw}`, en: `Group ${pad2(g.num)}: ${g.name_en}` }
    },
    formulas
  };
  const fn2 = `kundi${pad2(g.num)}_${g.id}.json`;
  const fp = path.join(OUT_DIR, fn2);
  fs.writeFileSync(fp, JSON.stringify(file, null, 2), 'utf8');
  console.log('Wrote', fn2, '→', formulas.length, 'formulas');
});
console.log('DONE. All groups written.');
