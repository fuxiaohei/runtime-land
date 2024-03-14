const projectName = document.getElementById("editor-container").getAttribute("data-x-name");
const projectSaveUrl = `/playground/${projectName}`;
const projectCheckUrl = `/playground/${projectName}/check`;

function setWaiting() {
    const btn = document.getElementById("main-btn");
    btn.setAttribute("disabled", true);
    btn.querySelector(".waiting-status").classList.remove("d-none");
    btn.querySelector(".submit-status").classList.add("d-none");
    btn.querySelector(".msg-text").innerText = "uploading";
    window.deployStatus = "waiting";
}

function setWaitingMessage(text) {
    const btn = document.getElementById("main-btn");
    btn.querySelector(".msg-text").innerText = text;
}

function setWaitingDone() {
    const btn = document.getElementById("main-btn");
    btn.querySelector(".submit-status").classList.remove("d-none");
    btn.querySelector(".waiting-status").classList.add("d-none");
    btn.removeAttribute("disabled");
    window.deployStatus = "";
}

function showMessage(text) {
    document.getElementById('msg-toast-text').innerText = text;
    const toast = document.getElementById('msg-toast');
    const toastBootstrap = bootstrap.Toast.getOrCreateInstance(toast);
    toastBootstrap.show();
    setTimeout(() => {
        toastBootstrap.hide();
    }, 5000);
}

async function handleCheck() {
    let resp = await fetch(projectCheckUrl);
    if (resp.ok) {
        let data = await resp.json();
        if (data.deploy_status == "failed") {
            showMessage(data.deploy_message);
            setWaitingDone();
            return;
        }
        if (data.deploy_status == "success") {
            setWaitingDone();
            return;
        }
        setWaitingMessage(data.deploy_status);
    }
}

(() => {
    window.deployStatus = "";
    setInterval(async function () {
        if (window.deployStatus == "waiting") {
            // await handleCheck();
        }
    }, 1000)
})();

(() => {
    require.config({ paths: { vs: 'https://cdn.jsdelivr.net/npm/monaco-editor@0.46.0/min/vs' } });
    require(['vs/editor/editor.main'], function () {
        document.getElementById('editor-loading').classList.add('d-none');
        let container = document.getElementById('editor-container');
        container.classList.remove('d-none');
        let code = document.getElementById('code').textContent;
        var editor = monaco.editor.create(container, {
            automaticLayout: true,
            value: code,
            language: 'javascript',
            scrollBeyondLastLine: false,
            scrollBeyondLastColumn: 0,
            scrollbar: {
                useShadows: false,
                vertical: 'hidden',
                horizontal: 'hidden',
            },
            minimap: { enabled: false },
            overviewRulerLanes: 0,
            fontSize: "14px",
        });
        const btn = document.getElementById("main-btn");
        btn.addEventListener("click", async function (e) {
            let data = new URLSearchParams();
            data.append("source", editor.getValue());
            let response = await fetch(projectSaveUrl, {
                method: "POST",
                body: data
            });
            if (response.ok) {
                setWaiting();
            } else {
                alert("Failed to save!");
            }
        })
    });
})();

(() => {
    // convert data-x-timeago to local time string
    const xTimeElements = document.querySelectorAll('[data-x-timeago]');
    xTimeElements.forEach((element) => {
        const xTime = element.getAttribute('data-x-timeago');
        const date = new Date(xTime);
        element.innerText = timeago.format(date, "en_US");
    });
})();