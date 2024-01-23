function ajaxForm(selector, action, alert_prefix) {
    const sform = document.getElementById(selector);
    sform.addEventListener('submit', async (e) => {
        e.preventDefault();
        const formData = new FormData(sform);
        const response = await fetch(action, {
            method: 'POST',
            body: new URLSearchParams(formData),
        });
        if (response.ok) {
            document.getElementById(alert_prefix + '-alert-success').classList.remove('d-none');
            document.getElementById(alert_prefix + '-alert-fail').classList.add('d-none');
            setTimeout(() => {
                document.getElementById(alert_prefix + '-alert-success').classList.add('d-none');
            }, 3000);
        } else {
            let text = await response.text() || response.statusText;
            document.getElementById(alert_prefix + '-alert-success').classList.add('d-none');
            document.getElementById(alert_prefix + '-alert-fail').classList.remove('d-none');
            document.getElementById(alert_prefix + '-alert-fail').innerText = "Error: " + text;
            setTimeout(() => {
                document.getElementById(alert_prefix + '-alert-fail').classList.add('d-none');
            }, 3000);
        }
    });
}