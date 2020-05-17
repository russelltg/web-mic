
window.onload = () => {
    const status = document.getElementById("status");
    const error = document.getElementById("error");

    status.innerText = "Loading...";


    const wsPromise = new Promise((resolve, reject) => {
        const ws = new WebSocket(`wss://${window.location.host}/audio`);
        ws.onopen = () => resolve(ws);
    });
    const mediaPromise = navigator.mediaDevices.getUserMedia({ audio: true }).then(stream => {
        const ctx = new AudioContext();
        const source = ctx.createMediaStreamSource(stream);

        return ctx.audioWorklet.addModule('dist/worklet.js').then(() => [source, ctx]);
    });

    Promise.all([wsPromise, mediaPromise]).then(([ws, [source, ctx]]) => {
        status.innerText = "Initialized!";

        const capture = new AudioWorkletNode(ctx, 'capture-processor', { numberOfInputs: 1, numberOfOutputs: 0 });
        capture.port.onmessage = event => {
            ws.send(event.data);
        };
        source.connect(capture);
    });

    window.onerror = (msg, src, line, col, err) => {
        error.innerText += `${msg} ${src}:${line}:${col} ${err.toString()}\n`;
    }
};
