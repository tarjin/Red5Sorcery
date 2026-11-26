const events=new Set(['app_loaded','cloak_completed','decloak_completed','cloak_failed','decloak_failed']);
const tiers=new Set(['small','medium','large']);
const text=(v,max)=>typeof v==='string'&&v.trim().length>0&&v.length<=max;
const exact=(o,keys)=>o&&typeof o==='object'&&!Array.isArray(o)&&Object.keys(o).every(k=>keys.includes(k));
export default {async fetch(req,env){
 const origin=req.headers.get('Origin');const allowed=(env.ALLOWED_ORIGINS||'').split(',').map(x=>x.trim()).filter(Boolean);
 const cors=origin&&allowed.includes(origin)?{'Access-Control-Allow-Origin':origin,'Vary':'Origin','Access-Control-Allow-Methods':'POST, OPTIONS','Access-Control-Allow-Headers':'Content-Type'}:{};
 const json=(status,obj)=>new Response(JSON.stringify(obj),{status,headers:{...cors,'Content-Type':'application/json','Cache-Control':'no-store'}});
 if(!origin||!allowed.includes(origin))return json(403,{ok:false});
 if(req.method==='OPTIONS')return new Response(null,{status:204,headers:cors});
 if(req.method!=='POST')return json(405,{ok:false});
 const path=new URL(req.url).pathname;
 if(!['/usage','/feedback','/licensing'].includes(path))return json(404,{ok:false});
 if(!env.RATE_LIMITER||!env.DB)return json(503,{ok:false});
 try{
  // IP used only as ephemeral rate-limit key, never inserted into D1.
  const ip=req.headers.get('CF-Connecting-IP');if(!ip)return json(403,{ok:false});
  const rate=await env.RATE_LIMITER.limit({key:path+':'+ip});if(!rate.success)return json(429,{ok:false});
  if(!req.headers.get('content-type')?.startsWith('application/json'))return json(415,{ok:false});
  const reader=req.body?.getReader();if(!reader)return json(400,{ok:false});let len=0,parts=[];
  while(true){const {done,value}=await reader.read();if(done)break;len+=value.byteLength;if(len>24000){await reader.cancel();return json(413,{ok:false})}parts.push(value)}
  const bytes=new Uint8Array(len);let offset=0;for(const part of parts){bytes.set(part,offset);offset+=part.length}
  let d;try{d=JSON.parse(new TextDecoder('utf-8',{fatal:true}).decode(bytes))}catch{return json(400,{ok:false})}
  if(path==='/usage'){
   if(!exact(d,['event','tier'])||!events.has(d.event)||(d.tier!==undefined&&(!tiers.has(d.tier)||!d.event.startsWith('cloak_'))))return json(400,{ok:false});
   const country=typeof req.cf?.country==='string'&&/^[A-Z]{2}$/.test(req.cf.country)?req.cf.country:null;
   await env.DB.prepare('INSERT INTO usage_events(event,tier,country) VALUES(?,?,?)').bind(d.event,d.tier??null,country).run();
  }else if(path==='/feedback'){
   if(!exact(d,['score','comment'])||!Number.isInteger(d.score)||d.score<0||d.score>10||!text(d.comment,2000)||d.comment.trim().length<15)return json(400,{ok:false});
   await env.DB.prepare('INSERT INTO nps_feedback(score,comment) VALUES(?,?)').bind(d.score,d.comment.trim()).run();
  }else{
   if(!exact(d,['name','email','organization','message'])||!text(d.name,150)||!text(d.email,254)||!/^\S+@[^\s@]+\.[^\s@]+$/.test(d.email)||!text(d.message,4000)||(d.organization!==undefined&&(typeof d.organization!=='string'||d.organization.length>200)))return json(400,{ok:false});
   await env.DB.prepare('INSERT INTO licensing_inquiries(name,email,organization,message) VALUES(?,?,?,?)').bind(d.name.trim(),d.email.trim(),d.organization?.trim()||null,d.message.trim()).run();
  }
  return json(201,{ok:true});
 }catch{return json(503,{ok:false})}
}};
