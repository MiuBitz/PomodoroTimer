const { chromium } = require(process.env.POMODORO_NODE_MODULES ? process.env.POMODORO_NODE_MODULES + '/playwright' : 'playwright');
const path = require('path');
const assert = require('assert');
(async () => {
  const browser = await chromium.launch({channel:'msedge',headless:true});
  const page = await browser.newPage({viewport:{width:460,height:620}});
  const errors=[];page.on('pageerror',e=>errors.push(e.message));
  await page.addInitScript(() => {
    let config={work_minutes:25,break_minutes:5,cycles:4,start_with_windows:false,notifications_enabled:true,sound_enabled:true,minimize_to_tray:true,theme_mode:'dark',color_preset:'blue'};
    let state={running:false,paused:false,completed:false,phase:'work',cycle:0,total:4,remaining_seconds:1500,progress:0};
    window.__test={config:()=>config,complete:()=>{state={...state,running:false,completed:true,remaining_seconds:0,progress:1};}};
    window.__TAURI__={core:{invoke:async(command,args)=>{
      if(command==='get_settings')return {...config};
      if(command==='get_state')return {...state};
      if(command==='save_settings'){config=args.settings;return;}
      if(command==='timer_action'){
        if(args.action==='start')state={...state,running:true,paused:false,completed:false,cycle:1,total:args.cycles};
        if(args.action==='pause')state.paused=true;
        if(args.action==='resume')state.paused=false;
        if(args.action==='reset')state={...state,running:false,paused:false,completed:false,remaining_seconds:config.work_minutes*60,progress:0};
        return {...state};
      }
      throw Error(command);
    }}};
  });
  await page.goto('file:///'+path.resolve('ui/index.html').replaceAll('\\','/'));
  await page.waitForFunction(()=>!document.getElementById('start').disabled);
  assert.equal(await page.locator('#time').textContent(),'25:00');
  await page.screenshot({path:'target/ui-dark.png'});
  for (const dismissal of ['cancel', 'close', 'escape']) {
    await page.locator('#settings').click();
    await page.locator('[name=theme_mode]').selectOption('light');
    await page.locator('[name=color_preset]').selectOption('sage');
    assert.equal(await page.evaluate(()=>document.documentElement.dataset.theme),'light');
    assert.equal(await page.evaluate(()=>document.documentElement.dataset.preset),'sage');
    assert.equal(await page.evaluate(()=>window.__test.config().theme_mode),'dark');
    if (dismissal === 'escape') await page.keyboard.press('Escape');
    else await page.locator('#'+dismissal).click();
    await page.waitForFunction(()=>document.documentElement.dataset.theme==='dark');
    assert.equal(await page.evaluate(()=>document.documentElement.dataset.preset),'blue');
  }
  await page.locator('#cycles').fill('0');await page.locator('#start').click();
  assert.match(await page.locator('#error').textContent(),/whole number/);
  await page.locator('#cycles').fill('2');await page.locator('#start').click();
  assert(await page.locator('#cycles').isDisabled());
  await page.locator('#pause').click();assert.equal(await page.locator('#pause').textContent(),'Resume');
  await page.locator('#pause').click();assert.equal(await page.locator('#pause').textContent(),'Pause');
  assert.equal(await page.locator('#start svg').count(),1);
  assert.equal(await page.locator('#pause svg').count(),1);
  await page.locator('#settings').click();
  await page.locator('[name=work_minutes]').fill('30');
  await page.locator('[name=theme_mode]').selectOption('light');
  await page.locator('[name=color_preset]').selectOption('sage');
  assert.equal(await page.evaluate(()=>document.documentElement.dataset.theme),'light');
  assert.equal(await page.evaluate(()=>document.documentElement.dataset.preset),'sage');
  await page.screenshot({path:'target/ui-settings.png'});
  await page.locator('#save').click();
  assert.equal(await page.evaluate(()=>document.documentElement.dataset.theme),'light');
  assert.equal(await page.locator('#time').textContent(),'25:00');
  await page.evaluate(()=>window.__test.complete());
  await page.waitForFunction(()=>document.getElementById('phase').textContent==='All cycles complete');
  await page.waitForTimeout(600);
  assert.equal(await page.locator('#time').textContent(),'00:00');
  await page.locator('#reset').click();
  assert.equal(await page.locator('#time').textContent(),'30:00');
  await page.setViewportSize({width:360,height:510});
  assert(await page.evaluate(()=>document.documentElement.scrollWidth<=innerWidth));
  await page.locator('#settings').click();
  assert(await page.locator('#save').isVisible());
  assert.deepEqual(errors,[]);
  console.log('Frontend passed: live appearance preview, cancel/close/Escape restoration, saving, timer controls, narrow layout.');
  await browser.close();
})().catch(e=>{console.error(e);process.exit(1);});
