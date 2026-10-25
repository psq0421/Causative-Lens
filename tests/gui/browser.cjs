// Synthetic logs exercise the real daemon. No vendor compatibility is inferred.
const {chromium} = require('playwright');
const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const os = require('node:os');
const net = require('node:net');
const http = require('node:http');
const {spawn} = require('node:child_process');
const root = path.resolve(__dirname, '../..');
const frontend = path.join(root, 'crates/clens-gui/frontend');
const out = path.join(root, 'dist/gui-v0.1.1');
fs.mkdirSync(out, {recursive:true});
const folder = fs.mkdtempSync(path.join(os.tmpdir(), 'causativelens-gui-'));
const log = path.join(folder, 'trace.jsonl');
fs.writeFileSync(log, '');
const pipe = process.platform === 'win32' ? '\\\\.\\pipe\\causativelens-rpc' : '/tmp/causativelens-rpc';
const sleep = ms => new Promise(resolve=>setTimeout(resolve,ms));
function call(method, params={}) {
  return new Promise((resolve,reject)=>{
    const socket=net.createConnection(pipe); let data='',sent=false;
    socket.setTimeout(8000,()=>socket.destroy(new Error('RPC timeout')));
    socket.on('connect',()=>{sent=true;socket.write(JSON.stringify({id:1,method,params})+'\n');});
    socket.on('data',chunk=>{
      data+=chunk.toString('utf8');
      if(data.includes('\n')) {
        socket.end();
        try {const response=JSON.parse(data.split('\n')[0]);response.error?reject(new Error(JSON.stringify(response.error))):resolve(response.result);} catch(e){reject(e);}
      }
    });
    socket.on('error',async error=>{
      if(!sent && ['EBUSY','ENOENT','ECONNREFUSED'].includes(error.code)) {
        reject(Object.assign(error,{retryable:true}));
      } else reject(error);
    });
  });
}
async function rpc(method,params={}) {
  for(let i=0;i<80;i++) {
    try{return await call(method,params);}catch(e){if(!e.retryable||i===79)throw e;await sleep(25);}
  }
}
let daemon,browser,server;
const sessionIds=[];
(async()=>{
  try {await call('hello',{protocol_version:1,mode:'observe'});} catch {
    const binary=process.env.CAUSATIVELENS_TEST_DAEMON||path.join(root,'target/debug/clens-daemon'+(process.platform==='win32'?'.exe':''));
    daemon=spawn(binary,[],{windowsHide:true,stdio:'ignore'});
    daemon.on('error',error=>{throw error;});
    await rpc('hello',{protocol_version:1,mode:'observe'});
  }
  server=http.createServer(async(req,res)=>{
    try {
      if(req.url==='/rpc'&&req.method==='POST') {
        let data='';for await(const chunk of req){data+=chunk;if(data.length>65536)throw new Error('oversize');}
        const input=JSON.parse(data), result=await rpc(input.method,input.params);
        if(input.method==='monitor.start') sessionIds.push(result.session_id);
        res.setHeader('Content-Type','application/json');res.end(JSON.stringify(result));return;
      }
      const file=path.join(frontend,req.url==='/'?'index.html':req.url.slice(1));
      if(!file.startsWith(frontend+path.sep)){res.writeHead(403).end();return;}
      const mime={'.html':'text/html; charset=utf-8','.css':'text/css','.js':'text/javascript','.mjs':'text/javascript'};
      res.setHeader('Content-Type',mime[path.extname(file)]||'text/plain');
      res.end(fs.readFileSync(file));
    }catch(e){res.writeHead(500,{'Content-Type':'application/json'});res.end(JSON.stringify({error:e.message}));}
  });
  await new Promise(resolve=>server.listen(0,'127.0.0.1',resolve));
  browser=await chromium.launch({headless:true});
  const context=await browser.newContext({viewport:{width:1440,height:1000},locale:'zh-CN',colorScheme:'light'});
  await context.addInitScript(({folder})=>{
    window.__TAURI__={core:{invoke:async(command,args={})=>{
      if(command==='pick_monitor_folder')return folder;
      const input=command==='connect_daemon'?{method:'hello',params:{protocol_version:1,mode:'observe'}}:args;
      const response=await fetch('/rpc',{method:'POST',headers:{'Content-Type':'application/json'},body:JSON.stringify(input)});
      const result=await response.json();if(!response.ok)throw new Error(result.error);return result;
    }}};
  },{folder});
  const page=await context.newPage(),errors=[];
  page.on('pageerror',e=>errors.push(e.message));
  await page.goto('http://127.0.0.1:'+server.address().port);
  await page.waitForSelector('#connection-status.connected');
  await page.waitForFunction(()=>!document.querySelector('#runtime-select').disabled);
  assert.equal(await page.locator('#runtime-select option').count(),21);
  await page.screenshot({path:path.join(out,'01-listen-light.png'),fullPage:true});
  await page.locator('[data-runtime="openclaw"]').click();
  await page.locator('#software-next').click();
  await page.locator('#browse-folder').click();
  assert.equal(await page.locator('#monitor-folder').inputValue(),folder);
  await page.locator('#include-profile').uncheck();
  await page.locator('#probe-folder').click();
  await page.waitForSelector('[data-step="2"]:not(.hidden)');
  // A locale switch must preserve the probed configuration.
  await page.locator('#language-select').selectOption('zh-TW');
  await page.locator('#start-monitor').click();
  await page.waitForFunction(()=>document.querySelector('#live-label').textContent.includes('等待'));
  const appended=Date.now();
  fs.appendFileSync(log,[
    {tool_name:'read',args:{path:'C:\\demo\\config.yaml'}},
    {tool_name:'write',args:{path:'C:\\demo\\result.txt'}},
    {tool_name:'connect',args:{target:'api.example.com:443'}},
    {text:'<img src=x onerror="window.unexpectedXss=true">'}
  ].map(x=>JSON.stringify(x)).join('\n')+'\n');
  await page.waitForFunction(()=>Number(document.querySelector('#stat-records').textContent)>=4);
  const visibleLatency=Date.now()-appended;
  assert.ok(visibleLatency<3000,`append latency ${visibleLatency}ms`);
  assert.equal(await page.locator('#start-monitor').isDisabled(),true);
  await page.locator('#view-timeline').click();
  const count=async()=>page.locator('.timeline-row').count();
  await page.waitForFunction(()=>document.querySelectorAll('.timeline-row').length>=7);
  const all=await count();
  for(const [filter,expected] of [['assertions',3],['files',2],['network',1]]) {
    await page.locator(`[data-filter="${filter}"]`).click();
    assert.equal(await count(),expected,filter);
    assert.equal(await page.locator(`[data-filter="${filter}"]`).getAttribute('aria-pressed'),'true');
    assert.ok((await page.locator('.timeline-row').evaluateAll(nodes=>nodes.map(n=>n.dataset.category))).every(c=>c===filter));
  }
  await page.locator('[data-filter="all"]').click();assert.equal(await count(),all);
  await page.locator('#timeline-search').fill('result.txt');assert.equal(await count(),2);
  await page.locator('.timeline-row').first().focus();await page.keyboard.press('Enter');
  await page.waitForSelector('#detail-overlay:not(.hidden)');
  assert.ok((await page.locator('#detail-content').textContent()).includes('result.txt'));
  await page.keyboard.press('Escape');
  await page.locator('#timeline-search').fill('');
  assert.equal(await page.locator('#timeline-rows img').count(),0);
  assert.equal(await page.evaluate(()=>Boolean(window.unexpectedXss)),false);
  await page.screenshot({path:path.join(out,'02-timeline-light.png'),fullPage:true});
  await page.locator('#theme-toggle').click();
  await page.locator('#language-select').selectOption('en-US');
  assert.equal(await page.locator('[data-i18n="filter.network"]').textContent(),'Network');
  await page.screenshot({path:path.join(out,'03-timeline-dark.png'),fullPage:true});
  await page.locator('#timeline-setup').click();
  await page.locator('#stop-monitor').click();
  await page.waitForFunction(()=>document.querySelector('#live-label').textContent.startsWith('Stopped')&&document.querySelector('#start-monitor').disabled===false);
  assert.equal(await page.locator('#start-monitor').isEnabled(),true);
  const read=await page.locator('#stat-records').textContent();
  fs.appendFileSync(log,JSON.stringify({text:'after stop'})+'\n');await sleep(700);
  assert.equal(await page.locator('#stat-records').textContent(),read);
  await page.reload();await page.waitForSelector('#connection-status.connected');
  await page.waitForSelector('#active-monitor:not(.hidden)');
  assert.equal(await page.locator('#stop-monitor').isDisabled(),true);
  await page.locator('#language-select').selectOption('zh-CN');
  await page.screenshot({path:path.join(out,'04-listen-dark.png'),fullPage:true});
  assert.equal(await page.evaluate(()=>document.documentElement.scrollWidth<=innerWidth),true);
  assert.deepEqual(errors,[]);
  const report={status:'pass',kind:'synthetic fixtures + real daemon RPC',append_to_gui_ms:visibleLatency,filters:{all,assertions:3,files:2,network:1},locales:['zh-CN','zh-TW','en-US'],screenshots:4,limitations:'Not a real-vendor, ETW, native picker, FPS or RSS test.'};
  fs.writeFileSync(path.join(out,'browser-report.json'),JSON.stringify(report,null,2));console.log(JSON.stringify(report));
})().catch(e=>{console.error(e);process.exitCode=1;}).finally(async()=>{
  if(browser)await browser.close();
  for(const id of sessionIds)await rpc('session.close',{session_id:id}).catch(()=>{});
  if(server)await new Promise(resolve=>server.close(resolve));
  if(daemon)daemon.kill();
  fs.rmSync(folder,{recursive:true,force:true});
});
