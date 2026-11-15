const fs=require('node:fs'),path=require('node:path'),crypto=require('node:crypto');
const root=path.resolve('.local/native-internal-annotations-20261115/A001');
const hash=b=>crypto.createHash('sha256').update(b).digest('hex');
const binding=JSON.parse(fs.readFileSync(path.join(root,'ROOT-BINDING.json')));
if(hash(fs.readFileSync(path.join(root,'ROOT-BINDING.json')))!=='ff8a6fd50d101fd6ad5c119194184a32410b9f0e3b01d337f07a3c7b7e00312e')throw Error('binding mismatch');
if(hash(fs.readFileSync(path.join(root,'ROOT-COMPLETION-RECEIPT-CLARIFICATION.json')))!=='4d3cc2c0fca2ecc8b644867bf36772b71e80245c7f923e7b5021e561bca71cdc')throw Error('clarification mismatch');
for(const [name,v] of Object.entries(binding.source_inputs)){const h=typeof v==='string'?v:v.sha256;if(hash(fs.readFileSync(path.join(binding.source_worktree,name)))!==h)throw Error('source changed:'+name);}
const controller=require(binding.controller).createController(binding.controller_configuration);
const recorder=require(binding.recorder).createRecorder(root);
const clock=controller.startClock();
const id=recorder.before('bootstrap-inventory','getState',{args:[{emit:false}],evidenceKind:'native',purpose:'documentation/inventory only; no target app control'});
fs.writeFileSync(path.join(root,'BOOTSTRAP-BEFORE.json'),JSON.stringify({id,clock,status:controller.status()},null,2)+'\n',{flag:'wx'});
console.log(JSON.stringify({id,clock,status:controller.status()}));
