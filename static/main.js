
window.onload = () => {
    const status = document.getElementById("status");
    const error = document.getElementById("error");

    navigator.mediaDevices.getUserMedia({ audio: true }).then(stream => {
        status.innerText += "Got audio device\n";

        const ctx = new AudioContext();
        const source = ctx.createMediaStreamSource(stream);

        const ws = new WebSocket(`wss://${window.location.host}/audio`);
        ws.onopen = () => {
            ctx.audioWorklet.addModule('dist/worklet.js').then(() => {
                status.innerText += "Got worklet\n";
                const capture = new AudioWorkletNode(ctx, 'capture-processor', { numberOfInputs: 1, numberOfOutputs: 0 });
                capture.port.onmessage = event => {
                    ws.send(event.data); // just send one channel
                };
                source.connect(capture);
            });
        };
    }).catch(e => error.innerText += e.toString());

    if ('wakeLock' in navigator) {
        const wl = navigator.wakeLock.request('screen').then();
    }

    window.onerror = (msg, src, line, col, err) => {
        error.innerText += `${msg} ${src}:${line}:${col} ${err.toString()}\n`;
    }
};
