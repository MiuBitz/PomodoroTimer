const { chromium } = require(process.env.POMODORO_NODE_MODULES ? process.env.POMODORO_NODE_MODULES + '/playwright' : 'playwright');
const {spawn} = require('child_process');
const fs = require('fs'), path = require('path'), assert = require('assert');
(async()=>{
  const appdata = fs.mkdtempSync(path.resolve('target/native-test-'));
  const child = spawn(path.resolve(process.argv[2] || 'dist/PomodoroTimer.exe'),['--minimized'],{
    windowsHide:true,env:{...process.env,APPDATA:appdata,WEBVIEW2_USER_DATA_FOLDER:path.join(appdata,'WebView2'),WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS:'--remote-debugging-port=9437'}
  });
  let browser;
  try {
    child.on('error',e=>console.error(e));
    let endpoint;
    for(let i=0;i<60;i++){
      if(child.exitCode!==null)throw Error('Application exited early: '+child.exitCode);
      try {const r=await fetch('http://127.0.0.1:9437/json/version');endpoint=(await r.json()).webSocketDebuggerUrl;break;} catch {}
      await new Promise(r=>setTimeout(r,500));
    }
    assert(endpoint,'WebView2 debugging endpoint unavailable');
    browser=await chromium.connectOverCDP(endpoint);
    const page=browser.contexts()[0].pages()[0];
    await page.waitForFunction(()=>window.__TAURI__ && !document.getElementById('start').disabled);
    const config=await page.evaluate(()=>window.__TAURI__.core.invoke('get_settings'));
    assert.equal(config.work_minutes,25);
    assert.equal(await page.locator('#settings svg').count(),1);
    if (process.env.POMODORO_CAPTURE) await page.screenshot({path:'docs/assets/app-dark.png'});
    await page.locator('#settings').click();
    if (process.env.POMODORO_CAPTURE) await page.screenshot({path:'docs/assets/settings-dark.png'});
    await page.locator('[name=theme_mode]').selectOption('light');
    await page.locator('[name=color_preset]').selectOption('sage');
    assert.equal(await page.evaluate(()=>document.documentElement.dataset.theme),'light');
    assert.equal(await page.evaluate(()=>document.documentElement.dataset.preset),'sage');
    assert.equal((await page.evaluate(()=>window.__TAURI__.core.invoke('get_settings'))).theme_mode,'dark');
    await page.locator('#cancel').click();
    await page.waitForFunction(()=>document.documentElement.dataset.theme==='dark');
    assert.equal(await page.evaluate(()=>document.documentElement.dataset.preset),'blue');
    await page.locator('#cycles').fill('2');await page.locator('#start').click();
    await page.waitForFunction(()=>document.getElementById('cycles').disabled);
    let state=await page.evaluate(()=>window.__TAURI__.core.invoke('get_state'));
    assert(state.running);assert.equal(state.total,2);
    await page.locator('#pause').click();
    await page.waitForFunction(()=>document.getElementById('pause').textContent==='Resume');
    state=await page.evaluate(()=>window.__TAURI__.core.invoke('get_state'));
    assert(state.paused);
    await page.locator('#settings').click();
    await page.locator('[name=work_minutes]').fill('30');await page.locator('#save').click();
    await page.waitForFunction(()=>!document.getElementById('dialog').open);
    state=await page.evaluate(()=>window.__TAURI__.core.invoke('get_state'));
    assert(state.remaining_seconds<=1500);assert(state.remaining_seconds>1490);
    await page.locator('#reset').click();
    await page.waitForFunction(()=>document.getElementById('time').textContent==='30:00');
    const saved=JSON.parse(fs.readFileSync(path.join(appdata,'PomodoroTimer/settings.json')));
    assert.equal(saved.work_minutes,30);
    assert.equal(saved.start_with_windows,false);
    console.log('Native Tauri passed: live preview without saving, cancel restoration, real IPC, timer controls, settings persistence.');
  } finally { if(browser) await browser.close().catch(()=>{}); child.kill(); }
})().catch(e=>{console.error(e);process.exit(1)});
