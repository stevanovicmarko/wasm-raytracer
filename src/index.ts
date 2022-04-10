import init, {make_image} from "../rust-wasm-raytracer/pkg";

init().then(() => {
    const renderSettings = document.getElementById('renderSettings') as HTMLElement;
    const widthInput = document.getElementById('canvasWidth') as HTMLInputElement;
    const heightInput = document.getElementById('canvasHeight') as HTMLInputElement;
    const raysPerPixel = document.getElementById(
        'raysPerPixel'
    ) as HTMLInputElement;
    const samplesLabel = document.getElementById('samplesLabel') as HTMLSpanElement;
    const sceneSelectButtons = document.getElementsByName(
        'scene-select'
    ) as NodeListOf<HTMLInputElement>;
    const samplingSelectButtons = document.getElementsByName(
        'sampler-select'
    ) as NodeListOf<HTMLInputElement>;
    const renderButton = document.getElementById(
        'renderButton'
    ) as HTMLButtonElement;
    const renderTime = document.getElementById('renderTime') as HTMLSpanElement;
    const canvas = document.getElementById('canvas') as HTMLCanvasElement;
    const ctx = canvas.getContext('2d') as CanvasRenderingContext2D;

    raysPerPixel.value = '16';
    widthInput.value = '800';
    heightInput.value = '500';

    samplesLabel.innerHTML = raysPerPixel.value;

    raysPerPixel.addEventListener('change', event => {
        samplesLabel.innerHTML = (event.target as HTMLInputElement).value;
    });

    widthInput.addEventListener('change', event => {
        canvas.width = parseInt((event.target as HTMLInputElement).value, 10);
    });

    heightInput.addEventListener('change', event => {
        canvas.height = parseInt((event.target as HTMLInputElement).value, 10);
    });

    let sceneType = 'predefined-scene';
    let samplingType = 'jittered-sampling';
    let width = canvas.width;
    let height = canvas.height;
    let numberOfSamples = parseInt(samplesLabel.innerText, 10);
    let t0 = 0;
    let t1 = 0;

    let preventRenderRequests = false;

    renderButton.addEventListener('click', event => {
        if (preventRenderRequests) {
            return;
        }

        sceneSelectButtons.forEach(radioButton => {
            if (radioButton.checked) {
                sceneType = radioButton.id;
            }
        });

        samplingSelectButtons.forEach(radioButton => {
            if (radioButton.checked) {
                samplingType = radioButton.id;
            }
        });

        width = canvas.width;
        height = canvas.height;
        numberOfSamples = parseInt(samplesLabel.innerText, 10);
        const isRandomScene = sceneType !== 'predefined-scene';
        const isJitteredSampling = samplingType === 'jittered-sampling';

        preventRenderRequests = true;
        renderSettings.style.pointerEvents = 'none';
        renderSettings.style.opacity = '0.2';

        renderTime.innerHTML = 'Rendering in progress...';

        setTimeout(() => {

            t0 = performance.now();
            const result = make_image(
                width,
                height,
                numberOfSamples,
                isRandomScene,
                isJitteredSampling
            );

            console.log("before render");
            const imageData = new ImageData(
                new Uint8ClampedArray(result.buffer),
                width,
                height
            );
            ctx.putImageData(imageData, 0, 0);
            console.log("after render")
            preventRenderRequests = false;
            renderSettings.style.pointerEvents = 'auto';
            renderSettings.style.opacity = '1.0';
            t1 = performance.now();
            renderTime.innerHTML = `Rendering completed in ${Number((t1 - t0) / 1000).toFixed(2)} seconds.`;
        }, 100);
    });

});
