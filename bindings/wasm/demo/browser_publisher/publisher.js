import init, {
  DomainParticipantFactory,
  DynamicTypeBuilderFactory,
  MemberDescriptor,
  TypeKind,
  TopicQos,
  DataReaderQos,
  HistoryQosPolicy,
  HistoryQosPolicyKind,
  StatusKind,
} from './pkg/dust_dds_wasm.js';

let dataWriter = null;
let sineReader = null;
let participantFactory = null;
let participant = null;
let currentMessageId = 1;
let publishedCount = 0;

// UI Elements - General & TextMessage
const signalingBadge = document.getElementById('signaling-badge');
const signalingText = document.getElementById('signaling-text');
const matchedBadge = document.getElementById('matched-badge');
const matchedText = document.getElementById('matched-text');
const messageInput = document.getElementById('message-input');
const publishBtn = document.getElementById('publish-btn');
const logContainer = document.getElementById('log-container');
const logEmpty = document.getElementById('log-empty');
const countDisplay = document.getElementById('count-display');
const clearBtn = document.getElementById('clear-btn');
const chips = document.querySelectorAll('.chip');

// UI Elements - Sine Wave Canvas
const sineCanvas = document.getElementById('sine-canvas');
const sineCtx = sineCanvas ? sineCanvas.getContext('2d') : null;
const sineValueDisplay = document.getElementById('sine-value');
const sineStatusBadge = document.getElementById('sine-status-badge');
const sineStatusText = document.getElementById('sine-status-text');

function setBadge(badgeEl, textEl, state, text) {
  if (!badgeEl || !textEl) return;
  badgeEl.className = `badge badge-${state}`;
  textEl.textContent = text;
}

function addLogEntry(id, message) {
  if (logEmpty) {
    logEmpty.style.display = 'none';
  }
  const now = new Date();
  const timeStr = now.toLocaleTimeString();

  const entry = document.createElement('div');
  entry.className = 'log-entry';
  entry.innerHTML = `
    <div>
      <span class="log-id">#${id}</span>
      <span class="log-msg">${escapeHtml(message)}</span>
    </div>
    <span class="log-time">${timeStr}</span>
  `;
  logContainer.prepend(entry);

  publishedCount++;
  countDisplay.textContent = publishedCount.toString();
}

function escapeHtml(text) {
  const div = document.createElement('div');
  div.textContent = text;
  return div.innerHTML;
}

// -----------------------------------------------------------------------------
// Sine Wave Canvas Plotting
// -----------------------------------------------------------------------------
function drawSinePlot(values) {
  if (!sineCanvas || !sineCtx) return;
  const width = sineCanvas.width;
  const height = sineCanvas.height;

  // Clear background
  sineCtx.fillStyle = '#0b1120';
  sineCtx.fillRect(0, 0, width, height);

  const padLeft = 45;
  const padRight = 30;
  const padTop = 25;
  const padBottom = 25;
  const plotWidth = width - padLeft - padRight;
  const plotHeight = height - padTop - padBottom;

  // Grid lines: +1.0, 0.0, -1.0
  const yZero = padTop + plotHeight / 2;
  const yPlus1 = padTop + plotHeight * 0.1;
  const yMinus1 = padTop + plotHeight * 0.9;

  sineCtx.lineWidth = 1;
  sineCtx.strokeStyle = '#1e293b';
  sineCtx.setLineDash([4, 4]);

  // +1 line
  sineCtx.beginPath();
  sineCtx.moveTo(padLeft, yPlus1);
  sineCtx.lineTo(width - padRight, yPlus1);
  sineCtx.stroke();

  // -1 line
  sineCtx.beginPath();
  sineCtx.moveTo(padLeft, yMinus1);
  sineCtx.lineTo(width - padRight, yMinus1);
  sineCtx.stroke();

  // 0 line (solid, slightly brighter)
  sineCtx.strokeStyle = '#334155';
  sineCtx.setLineDash([]);
  sineCtx.beginPath();
  sineCtx.moveTo(padLeft, yZero);
  sineCtx.lineTo(width - padRight, yZero);
  sineCtx.stroke();

  // Axis Labels
  sineCtx.fillStyle = '#64748b';
  sineCtx.font = '11px ui-monospace, monospace';
  sineCtx.textAlign = 'right';
  sineCtx.textBaseline = 'middle';
  sineCtx.fillText('+1.0', padLeft - 8, yPlus1);
  sineCtx.fillText(' 0.0', padLeft - 8, yZero);
  sineCtx.fillText('-1.0', padLeft - 8, yMinus1);

  if (!values || values.length === 0) {
    sineCtx.fillStyle = '#475569';
    sineCtx.textAlign = 'center';
    sineCtx.font = '13px sans-serif';
    sineCtx.fillText('Waiting for SineWave samples...', padLeft + plotWidth / 2, yZero);
    return;
  }

  // Map value [-1.0, 1.0] to Y coordinate
  const mapY = (val) => {
    const clamped = Math.max(-1.2, Math.min(1.2, val));
    return yZero - (clamped * (yZero - yPlus1));
  };

  // Map index across 10 slots
  const slotCount = Math.max(100, values.length);
  const xStep = plotWidth / (slotCount - 1);
  const getX = (idx) => padLeft + idx * xStep;

  // Draw gradient fill under curve
  sineCtx.beginPath();
  values.forEach((v, i) => {
    const x = getX(i);
    const y = mapY(v);
    if (i === 0) sineCtx.moveTo(x, y);
    else sineCtx.lineTo(x, y);
  });
  sineCtx.lineTo(getX(values.length - 1), yZero);
  sineCtx.lineTo(getX(0), yZero);
  sineCtx.closePath();

  const grad = sineCtx.createLinearGradient(0, yPlus1, 0, yMinus1);
  grad.addColorStop(0, 'rgba(56, 189, 248, 0.25)');
  grad.addColorStop(0.5, 'rgba(56, 189, 248, 0.05)');
  grad.addColorStop(1, 'rgba(56, 189, 248, 0.25)');
  sineCtx.fillStyle = grad;
  sineCtx.fill();

  // Draw connecting curve/line
  sineCtx.beginPath();
  values.forEach((v, i) => {
    const x = getX(i);
    const y = mapY(v);
    if (i === 0) sineCtx.moveTo(x, y);
    else sineCtx.lineTo(x, y);
  });
  sineCtx.strokeStyle = '#38bdf8';
  sineCtx.lineWidth = 2.5;
  sineCtx.lineJoin = 'round';
  sineCtx.lineCap = 'round';
  sineCtx.stroke();

  // Draw points
  values.forEach((v, i) => {
    const x = getX(i);
    const y = mapY(v);
    const isLatest = (i === values.length - 1);

    sineCtx.beginPath();
    sineCtx.arc(x, y, isLatest ? 5.5 : 3.5, 0, Math.PI * 2);
    sineCtx.fillStyle = isLatest ? '#34d399' : '#38bdf8';
    sineCtx.fill();
    sineCtx.lineWidth = 2;
    sineCtx.strokeStyle = '#0b1120';
    sineCtx.stroke();

    // Value text for latest point
    if (isLatest) {
      sineCtx.fillStyle = '#34d399';
      sineCtx.font = 'bold 11px ui-monospace, monospace';
      sineCtx.textAlign = 'center';
      const labelY = y < yZero ? y - 10 : y + 15;
      sineCtx.fillText((v >= 0 ? '+' : '') + v.toFixed(3), x, labelY);
    }
  });
}

// Initial empty canvas render
drawSinePlot([]);

async function publishCurrentInput() {
  const text = messageInput.value.trim();
  if (!text || !dataWriter) return;

  publishBtn.disabled = true;
  try {
    const id = currentMessageId++;
    await dataWriter.write({ id, message: text });
    addLogEntry(id, text);
    messageInput.value = '';
    messageInput.focus();
  } catch (err) {
    console.error('Failed to publish message:', err);
    alert('Failed to publish: ' + err);
  } finally {
    publishBtn.disabled = false;
  }
}

async function startDds() {
  try {
    setBadge(signalingBadge, signalingText, 'warning', 'Initializing WASM...');
    await init();

    const signalingUrl = 'ws://127.0.0.1:9001';
    setBadge(signalingBadge, signalingText, 'info', `Connecting to ${signalingUrl}...`);

    // 1. Create DomainParticipantFactory with signaling URL
    participantFactory = new DomainParticipantFactory(signalingUrl);

    // 2. Create Participant on domain 0
    participant = await participantFactory.createParticipant(0);
    setBadge(signalingBadge, signalingText, 'success', `Connected to ${signalingUrl}`);

    window.addEventListener('beforeunload', () => {
      if (participantFactory && participant) {
        participantFactory.deleteParticipant(participant).catch(() => {});
      }
    });

    // -------------------------------------------------------------------------
    // TextMessage Publisher Setup
    // -------------------------------------------------------------------------
    // 3. Define DynamicType for TextMessage: { id: u32 (@key), message: string }
    const idType = DynamicTypeBuilderFactory.getPrimitiveType(TypeKind.Uint32);
    const idMember = new MemberDescriptor('id', idType, 0, true, false);

    const messageType = DynamicTypeBuilderFactory.createStringType();
    const messageMember = new MemberDescriptor('message', messageType, 1, false, false);

    const textMsgBuilder = DynamicTypeBuilderFactory.createStructureType('TextMessage');
    textMsgBuilder.addMember(idMember);
    textMsgBuilder.addMember(messageMember);
    const textMsgDynamicType = textMsgBuilder.build();

    // 4. Create Topic, Publisher, and DataWriter for TextMessage
    const textMsgTopic = await participant.createTopic('TextMessage', textMsgDynamicType);
    const publisher = await participant.createPublisher();
    dataWriter = await publisher.createDataWriter(textMsgTopic);

    // Enable input elements
    messageInput.disabled = false;
    publishBtn.disabled = false;
    messageInput.focus();

    // -------------------------------------------------------------------------
    // SineWave Subscriber Setup (Topic without key, History KeepLast(10))
    // -------------------------------------------------------------------------
    // 5. Define DynamicType for SineWave: { value: f64 } (without key)
    const valueType = DynamicTypeBuilderFactory.getPrimitiveType(TypeKind.Float64);
    const valueMember = new MemberDescriptor('value', valueType, 0, false, false);

    const sineBuilder = DynamicTypeBuilderFactory.createStructureType('SineWave');
    sineBuilder.addMember(valueMember);
    const sineDynamicType = sineBuilder.build();

    // 6. Create Topic for SineWave with History KeepLast(10)
    const sineTopicQos = new TopicQos();
    sineTopicQos.setHistory(new HistoryQosPolicy(HistoryQosPolicyKind.KeepLast, 100));
    const sineTopic = await participant.createTopic('SineWave', sineDynamicType, sineTopicQos);

    // 7. Create Subscriber and DataReader for SineWave with History KeepLast(10)
    const subscriber = await participant.createSubscriber();

    const readerQos = new DataReaderQos();
    readerQos.setHistory(new HistoryQosPolicy(HistoryQosPolicyKind.KeepLast, 100));

    const sineListener = {
      onDataAvailable: async (reader) => {
        try {
          const samples = await reader.read(100);
          const values = [];
          for (const sample of samples) {
            if (sample.getSampleInfo().getValidData()) {
              values.push(sample.getData().value);
            }
          }
          if (values.length > 0) {
            drawSinePlot(values);
            const latest = values[values.length - 1];
            console.log(`[SineWave] Received ${values.length} sample(s), latest = ${latest.toFixed(4)}`);
            if (sineValueDisplay) {
              sineValueDisplay.textContent = (latest >= 0 ? '+' : '') + latest.toFixed(4);
            }
            if (sineStatusBadge && sineStatusText) {
              setBadge(
                sineStatusBadge,
                sineStatusText,
                'success',
                `Receiving (${values.length}/100 in history)`
              );
            }
          }
        } catch (err) {
          console.error('Error reading SineWave samples:', err);
        }
      },
      onSubscriptionMatched: (reader, status) => {
        console.log(`[SineWave] DataReader matched with publisher (count=${status.currentCount})`);
        if (status.currentCount > 0) {
          if (sineStatusBadge && sineStatusText) {
            setBadge(sineStatusBadge, sineStatusText, 'info', 'Publisher matched, awaiting samples...');
          }
        }
      },
    };

    sineReader = await subscriber.createDataReader(
      sineTopic,
      readerQos,
      sineListener,
      [StatusKind.DataAvailable, StatusKind.SubscriptionMatched]
    );

    // 8. Poll for matched DataWriter status
    setInterval(async () => {
      try {
        const status = await dataWriter.getPublicationMatchedStatus();
        if (status.currentCount > 0) {
          setBadge(
            matchedBadge,
            matchedText,
            'success',
            `Matched with DataReader (${status.currentCount} reader${status.currentCount > 1 ? 's' : ''})`
          );
        } else {
          setBadge(matchedBadge, matchedText, 'warning', 'Waiting for DataReader to connect...');
        }
      } catch (e) {
        // Ignored if entity not yet ready
      }
    }, 1000);
  } catch (err) {
    console.error('Failed to initialize Dust DDS:', err);
    setBadge(signalingBadge, signalingText, 'warning', `Error: ${err}`);
  }
}

// UI Event Handlers
document.getElementById('publish-form').addEventListener('submit', (e) => {
  e.preventDefault();
  publishCurrentInput();
});

publishBtn.addEventListener('click', (e) => {
  e.preventDefault();
  publishCurrentInput();
});

chips.forEach((chip) => {
  chip.addEventListener('click', () => {
    messageInput.value = chip.dataset.msg;
    publishCurrentInput();
  });
});

clearBtn.addEventListener('click', () => {
  logContainer.innerHTML = '<div class="log-empty" id="log-empty">No messages published yet.</div>';
  publishedCount = 0;
  countDisplay.textContent = '0';
});

// Expose publishMessage to window for automation/testing
window.publishMessage = async (msg) => {
  messageInput.value = msg;
  await publishCurrentInput();
};

// Check for auto-publish query parameter: ?auto=true&msg=...
const urlParams = new URLSearchParams(window.location.search);
if (urlParams.get('auto') === 'true') {
  const autoMsg = urlParams.get('msg') || 'Automated message from browser';
  const checkMatched = setInterval(async () => {
    if (!dataWriter) return;
    try {
      const status = await dataWriter.getPublicationMatchedStatus();
      if (status.currentCount > 0) {
        clearInterval(checkMatched);
        console.log('DataReader matched! Auto-publishing:', autoMsg);
        await window.publishMessage(autoMsg);
      }
    } catch (e) {}
  }, 500);
}

// Start DDS
startDds();
