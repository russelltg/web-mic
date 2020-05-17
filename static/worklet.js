
class CaptureWorklet extends AudioWorkletProcessor {
    constructor() {
        super();

        this.port.postMessage(sampleRate);
    }
    process(inputs, outputs, parameters) {
        this.port.postMessage(inputs[0][0])
        return true;
    }
}
registerProcessor('capture-processor', CaptureWorklet);
