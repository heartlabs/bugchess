#!/usr/bin/env node
const { chromium } = require('playwright');

const BASE_URL = process.env.BASE_URL || 'http://127.0.0.1:4001/index.htm';
const WAIT_MS = Number(process.env.WAIT_MS || 15000);

function installProbe(label) {
  const NativeWebSocket = window.WebSocket;
  window.WebSocket = function (...args) {
    const ws = new NativeWebSocket(...args);
    console.log(`[probe ${label}] ws:new ${args[0]}`);
    ws.addEventListener('open', () => console.log(`[probe ${label}] ws:open`));
    ws.addEventListener('message', (event) => {
      const payload = typeof event.data === 'string' ? event.data.slice(0, 400) : '[binary]';
      console.log(`[probe ${label}] ws:message ${payload}`);
    });
    ws.addEventListener('error', () => console.log(`[probe ${label}] ws:error`));
    ws.addEventListener('close', () => console.log(`[probe ${label}] ws:close`));

    const originalSend = ws.send.bind(ws);
    ws.send = (data) => {
      const payload = typeof data === 'string' ? data.slice(0, 400) : '[binary]';
      console.log(`[probe ${label}] ws:send ${payload}`);
      return originalSend(data);
    };

    return ws;
  };
  window.WebSocket.prototype = NativeWebSocket.prototype;

  const NativePeerConnection = window.RTCPeerConnection;
  window.RTCPeerConnection = function (...args) {
    const pc = new NativePeerConnection(...args);
    console.log(`[probe ${label}] pc:new`);

    pc.addEventListener('icegatheringstatechange', () => {
      console.log(`[probe ${label}] pc:iceGathering=${pc.iceGatheringState}`);
    });
    pc.addEventListener('iceconnectionstatechange', () => {
      console.log(`[probe ${label}] pc:iceConnection=${pc.iceConnectionState}`);
    });
    pc.addEventListener('signalingstatechange', () => {
      console.log(`[probe ${label}] pc:signaling=${pc.signalingState}`);
    });
    pc.addEventListener('icecandidate', (event) => {
      console.log(`[probe ${label}] pc:onicecandidate ${event.candidate ? 'candidate' : 'null'}`);
    });
    pc.addEventListener('datachannel', (event) => {
      const dc = event.channel;
      console.log(`[probe ${label}] pc:datachannel ${dc.label} state=${dc.readyState}`);
      dc.addEventListener('open', () => console.log(`[probe ${label}] dc:remote-open ${dc.label}`));
      dc.addEventListener('close', () => console.log(`[probe ${label}] dc:remote-close ${dc.label}`));
      dc.addEventListener('error', () => console.log(`[probe ${label}] dc:remote-error ${dc.label}`));
      dc.addEventListener('message', () => console.log(`[probe ${label}] dc:remote-message ${dc.label}`));
    });

    const originalCreateDataChannel = pc.createDataChannel.bind(pc);
    pc.createDataChannel = (...dcArgs) => {
      const dc = originalCreateDataChannel(...dcArgs);
      console.log(`[probe ${label}] dc:create ${dc.label} state=${dc.readyState}`);
      dc.addEventListener('open', () => console.log(`[probe ${label}] dc:open ${dc.label}`));
      dc.addEventListener('close', () => console.log(`[probe ${label}] dc:close ${dc.label}`));
      dc.addEventListener('error', () => console.log(`[probe ${label}] dc:error ${dc.label}`));
      dc.addEventListener('message', () => console.log(`[probe ${label}] dc:message ${dc.label}`));
      const originalSend = dc.send.bind(dc);
      dc.send = (data) => {
        console.log(`[probe ${label}] dc:send ${dc.label} state=${dc.readyState}`);
        return originalSend(data);
      };
      return dc;
    };

    for (const method of ['createOffer', 'createAnswer', 'setLocalDescription', 'setRemoteDescription', 'addIceCandidate']) {
      const original = pc[method].bind(pc);
      pc[method] = async (...methodArgs) => {
        console.log(`[probe ${label}] pc:${method}:called`);
        try {
          const result = await original(...methodArgs);
          console.log(`[probe ${label}] pc:${method}:resolved`);
          return result;
        } catch (error) {
          console.log(`[probe ${label}] pc:${method}:rejected ${error && error.message}`);
          throw error;
        }
      };
    }

    return pc;
  };
  window.RTCPeerConnection.prototype = NativePeerConnection.prototype;
}

async function main() {
  const browser1 = await chromium.launch({ headless: true });
  const browser2 = await chromium.launch({ headless: true });

  const context1 = await browser1.newContext();
  const context2 = await browser2.newContext();
  await context1.addInitScript(installProbe, 'host');
  await context2.addInitScript(installProbe, 'guest');

  const page1 = await context1.newPage();
  const page2 = await context2.newPage();

  for (const [page, label] of [[page1, 'host'], [page2, 'guest']]) {
    page.on('console', (msg) => console.log(`[${label} ${msg.type()}] ${msg.text()}`));
    page.on('pageerror', (error) => console.log(`[${label} pageerror] ${error.message}`));
    page.on('requestfailed', (request) => {
      console.log(`[${label} requestfailed] ${request.method()} ${request.url()} -> ${request.failure()?.errorText}`);
    });
  }

  await page1.goto(BASE_URL, { waitUntil: 'domcontentloaded' });
  await page1.getByText('Create Game', { exact: true }).click();
  const inviteUrl = (await page1.locator('#room_id_display').textContent())?.trim();
  if (!inviteUrl) throw new Error('No invite URL rendered');
  console.log(`INVITE ${inviteUrl}`);

  await page2.goto(inviteUrl, { waitUntil: 'domcontentloaded' });
  await page1.getByText('Join Game', { exact: true }).click();

  await page1.waitForTimeout(WAIT_MS);
  await page2.waitForTimeout(WAIT_MS);

  await context1.close();
  await context2.close();
  await browser1.close();
  await browser2.close();
}

main().catch((error) => {
  console.error(error);
  process.exit(1);
});
