const fs = require('fs');
const path = require('path');

const BASE = 'c:/Users/STEPHANO AMAN/Desktop/MTAALAMU SMART/mtaalamu-smart/data';
const FN_DIR = path.join(BASE, 'formulas_network');

let totalFiles = 0;
let totalFormulas = 0;
let allOK = true;

function check(file, label){
  try {
    const raw = fs.readFileSync(file, 'utf8');
    const d = JSON.parse(raw);
    totalFiles++;
    console.log('[OK]', label);
    return d;
  } catch(e) {
    allOK = false;
    console.log('[FAIL]', label, '→', e.message);
    return null;
  }
}

console.log('=== DIAGNOSIS VALIDATION ===');
const diag = check(path.join(BASE, 'diagnosis.json'), 'diagnosis.json');
if(diag){
  console.log('  Model count meta:', diag.meta.model_count);
  console.log('  Actual models:', Object.keys(diag.models).length, '(', Object.keys(diag.models).join(', '), ')');
  Object.entries(diag.models).forEach(([k,m])=>{
    const S = m.symptoms.length, C = m.causes.length, F = Object.keys(m.fixes).length;
    console.log(`  ${k}: symptoms=${S}, causes=${C}, fixes=${F}, likelihood cols=${m.likelihood[Object.keys(m.likelihood)[0]].length}`);
    if (m.likelihood[Object.keys(m.likelihood)[0]].length !== C) {
      console.log('    → WARNING: likelihood cols mismatch causes count');
      allOK = false;
    }
    const pSum = Object.values(m.priors).reduce((a,b)=>a+b,0);
    console.log(`    priors sum = ${pSum.toFixed(3)}`);
    if (Math.abs(pSum-1)>0.05) { console.log('    → WARNING: priors not ~1.0'); allOK=false; }
  });
}

console.log('\n=== PROBLEMS VALIDATION ===');
const prob = check(path.join(BASE, 'problems.json'), 'problems.json');
if(prob){
  console.log('  Total problems:', prob.problems.length);
  const tcounts = {};
  let cSumOK = 0;
  prob.problems.forEach(p=>{
    tcounts[p.trade] = (tcounts[p.trade]||0)+1;
    const s = Object.values(p.causes).reduce((a,b)=>a+b,0);
    if(Math.abs(s-1)<=0.01) cSumOK++;
    if(!p.id||!p.trade||!p.symptoms||!p.causes||!p.solution||p.time_min==null||p.cost_tzs==null||p.success_rate==null||!p.severity||p.cases==null){
      console.log('  → Missing fields on', p.id);
      allOK=false;
    }
  });
  console.log('  Causes sum OK (≤±0.01):', cSumOK, '/', prob.problems.length);
  console.log('  Per-trade counts (expect 20):');
  Object.entries(tcounts).forEach(([k,v])=>console.log('   ',k,'→',v, v===20?'✓':'✗'));
  const expected = ['computer','simu','umeme','maji','gari','hvac','cctv','solar','useremala','ujenzi','rangi','welding','pikipiki','jenereta','gas','roofing','pump','tailor','appliance','borehole','gate_motor'];
  expected.forEach(k=>{ if(!tcounts[k]){console.log('  → MISSING trade:', k); allOK=false;}});
}

console.log('\n=== FORMULAS NETWORK VALIDATION ===');
const expectedFiles = [
  'kundi01_signal.json','kundi02_antenna.json','kundi03_propagation.json','kundi04_link_budget.json',
  'kundi05_modulation.json','kundi06_traffic.json','kundi07_ran.json','kundi08_fiber.json',
  'kundi09_satellite.json','kundi10_nr5g.json','kundi11_subnetting.json','kundi12_bandwidth.json',
  'kundi13_latency.json','kundi14_routing.json','kundi15_qos.json','kundi16_wireless.json',
  'kundi17_security.json','kundi18_monitoring.json','kundi19_virtualization.json','kundi20_cloud.json'
];
const expectedCounts = [15,20,15,15,15,15,15,20,15,15,20,15,10,15,10,15,10,10,10,10];
const counts = [];
expectedFiles.forEach((f,i)=>{
  const fp = path.join(FN_DIR, f);
  if(!fs.existsSync(fp)){
    console.log('[MISSING]', f);
    allOK = false;
    return;
  }
  const d = check(fp, f);
  if(d){
    const n = d.formulas.length;
    counts.push(n);
    totalFormulas += n;
    const exp = expectedCounts[i];
    const ok = n===exp;
    console.log(`  formulas=${n}/${exp} ${ok?'✓':'✗ MISMATCH'}`);
    if(!ok) allOK=false;
    if(!d.group || !d.group.id || !d.group.code || !d.group.label || !d.group.label.sw || !d.group.label.en){
      console.log('  → Missing group fields'); allOK=false;
    }
    d.formulas.forEach(fm=>{
      if(!fm.id||!fm.group||!fm.inputs||!fm.outputs||!fm.steps||!fm.rules||!fm.test){
        console.log('  → Formula missing fields:', fm.id); allOK=false;
      }
    });
  }
});
console.log('\nTotal formulas in network:', totalFormulas, '(expect 285)');
if(totalFormulas!==285){ console.log('  → COUNT MISMATCH'); allOK=false; }

console.log('\n=== SUMMARY ===');
console.log('Total JSON files parsed OK:', totalFiles);
console.log('All checks passed:', allOK ? 'YES ✓' : 'NO ✗ (review WARNINGs above)');
process.exit(allOK ? 0 : 1);
