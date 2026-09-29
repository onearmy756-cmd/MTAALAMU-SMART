/**
 * MTAALAMU SMART — Agentic Voice (Kiswahili TTS)
 * Browser Web Speech API — offline-capable where OS supports sw/sw-TZ
 */
(function (global) {
  "use strict";

  var queue = [];
  var speaking = false;
  var preferredLang = "sw";

  function pickVoice(lang) {
    if (!global.speechSynthesis) return null;
    var voices = global.speechSynthesis.getVoices() || [];
    var want = (lang || "sw").toLowerCase();
    var exact = voices.filter(function (v) {
      return (v.lang || "").toLowerCase().indexOf(want) === 0;
    });
    if (exact.length) return exact[0];
    var soft = voices.filter(function (v) {
      return /sw|swa|kiswahili/i.test(v.lang + " " + v.name);
    });
    return soft[0] || null;
  }

  function speakText(text, opts) {
    opts = opts || {};
    if (!global.speechSynthesis || !text) return Promise.resolve();
    return new Promise(function (resolve) {
      try {
        global.speechSynthesis.cancel();
        var u = new SpeechSynthesisUtterance(String(text));
        u.lang = opts.lang || preferredLang || "sw";
        var v = pickVoice(u.lang);
        if (v) u.voice = v;
        u.rate = opts.rate || 0.95;
        u.pitch = opts.pitch || 1;
        u.onend = function () {
          speaking = false;
          resolve();
        };
        u.onerror = function () {
          speaking = false;
          resolve();
        };
        speaking = true;
        global.speechSynthesis.speak(u);
      } catch (e) {
        speaking = false;
        resolve();
      }
    });
  }

  function enqueue(texts) {
    if (!Array.isArray(texts)) texts = [texts];
    texts.forEach(function (t) {
      if (t && String(t).trim()) queue.push(String(t).trim());
    });
    drain();
  }

  function drain() {
    if (speaking || !queue.length) return;
    var next = queue.shift();
    speakText(next).then(function () {
      if (queue.length) setTimeout(drain, 250);
    });
  }

  function stop() {
    queue = [];
    if (global.speechSynthesis) global.speechSynthesis.cancel();
    speaking = false;
  }

  function setLang(lang) {
    preferredLang = lang || "sw";
  }

  // Chrome loads voices async
  if (global.speechSynthesis) {
    global.speechSynthesis.onvoiceschanged = function () {};
  }

  // Shiny custom message handlers
  if (global.Shiny && global.Shiny.addCustomMessageHandler) {
    global.Shiny.addCustomMessageHandler("av_speak", function (msg) {
      if (!msg) return;
      if (msg.lang) setLang(msg.lang);
      if (msg.stop) {
        stop();
        return;
      }
      if (msg.queue && msg.queue.length) enqueue(msg.queue);
      else if (msg.text) enqueue([msg.text]);
    });
  }

  global.MtaalamuVoice = {
    speak: speakText,
    enqueue: enqueue,
    stop: stop,
    setLang: setLang,
  };
})(window);
