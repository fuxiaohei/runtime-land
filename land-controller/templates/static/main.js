/*!
 * Color mode toggler for Bootstrap's docs (https://getbootstrap.com/)
 * Copyright 2011-2023 The Bootstrap Authors
 * Licensed under the Creative Commons Attribution 3.0 Unported License.
 */

(() => {
    'use strict'

    const getStoredTheme = () => localStorage.getItem('theme')
    const setStoredTheme = theme => localStorage.setItem('theme', theme)

    const getPreferredTheme = () => {
        const storedTheme = getStoredTheme()
        if (storedTheme) {
            return storedTheme
        }

        return window.matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light'
    }
    window.getPreferredTheme = getPreferredTheme;

    const setTheme = theme => {
        document.documentElement.setAttribute('data-bs-theme', theme)
        const links = document.querySelectorAll("link[title]");
        if (links.length > 0) {
            links.forEach((link) => {
                link.setAttribute('disabled', "disabled")
            });
            document.querySelector(`link[title="${theme}"]`).removeAttribute('disabled')
        }
    }

    setTheme(getPreferredTheme())

    const showActiveTheme = (theme, focus = false) => {
        const themeSwitcher = document.querySelector('#bd-theme')

        if (!themeSwitcher) {
            return
        }

        document.querySelectorAll('.bs-theme-current-icon').forEach(element => {
            element.classList.add('d-none')
        });
        const activeSvgIcon = document.querySelector(".bs-theme-current-icon-" + theme)
        activeSvgIcon.classList.remove('d-none')

        if (focus) {
            themeSwitcher.focus()
        }
    }

    window.matchMedia('(prefers-color-scheme: dark)').addEventListener('change', () => {
        const storedTheme = getStoredTheme()
        if (storedTheme !== 'light' && storedTheme !== 'dark') {
            setTheme(getPreferredTheme())
        }
    })

    const customElements = (theme) => {
        let btns = document.querySelectorAll(".custom-theme-btn")
        if (btns) {
            btns.forEach((btn) => {
                if (theme == "light") {
                    btn.classList.remove("btn-dark");
                    btn.classList.add("btn-secondary");
                } else {
                    btn.classList.remove("btn-secondary");
                    btn.classList.add("btn-dark");
                }
            });
        }
    }

    window.addEventListener('DOMContentLoaded', () => {
        showActiveTheme(getPreferredTheme())
        customElements(getPreferredTheme())
        let theme_btn = document.getElementById('bd-theme');
        if (theme_btn) {
            theme_btn.addEventListener('click', () => {
                const storedTheme = getStoredTheme()
                const currentTheme = storedTheme || getPreferredTheme()
                const newTheme = currentTheme === 'light' ? 'dark' : 'light'
                setStoredTheme(newTheme)
                customElements(newTheme)
                setTheme(newTheme)
                showActiveTheme(newTheme, true)
            });
        }
    })
})();

(() => {
    // enable tooltip
    const tooltipTriggerList = document.querySelectorAll('[data-bs-toggle="tooltip"]')
    const tooltipList = [...tooltipTriggerList].map(tooltipTriggerEl => new bootstrap.Tooltip(tooltipTriggerEl))
})();

(() => {
    // convert data-x-time to local time string
    const xTimeElements = document.querySelectorAll('[data-x-time]');
    xTimeElements.forEach((element) => {
        const xTime = element.getAttribute('data-x-time');
        const date = new Date(xTime);
        element.innerText = date.toLocaleString();
    });
    // convert data-x-timeago to local time string
    const xTimeElements2 = document.querySelectorAll('[data-x-timeago]');
    xTimeElements2.forEach((element) => {
        const xTime = element.getAttribute('data-x-timeago');
        const date = new Date(xTime);
        element.innerText = timeago.format(date, "en_US");
    });
})();

/// friendly_bytesize convert bytes to human readable size
function friendly_bytesize(v, with_byte_unit) {
    if (v < 0.1) {
        return 0
    }
    let bytes_units = ['iB', 'KiB', 'MiB', 'GiB', 'TiB', 'PiB', 'EiB', 'ZiB', 'YiB'];
    let units = ['', 'K', 'M', 'G', 'T', 'P', 'E', 'Z', 'Y'];
    let i = 0;
    while (v > 1000) {
        v /= 1000;
        i++;
    }
    v = v.toFixed(2);
    let u = with_byte_unit ? bytes_units[i] : units[i];
    if (u) {
        u = ' ' + u;
    }
    return `${v}${u}`
}


/// convert unixtimestamp to hour and minute, HH:MM
function unix2hour(v) {
    const dateObj = new Date(v)
    const hours = dateObj.getHours() >= 10 ? dateObj.getHours() : '0' + dateObj.getHours()
    const minutes = dateObj.getMinutes() < 10 ? dateObj.getMinutes() + '0' : dateObj.getMinutes()
    const UnixTimeToDate = hours + ':' + minutes
    if (window.traffic_period == "7d") {
        const month = dateObj.getMonth() + 1
        const days = dateObj.getDate() >= 10 ? dateObj.getDate() : '0' + dateObj.getDate()
        return month + '/' + days + ' ' + UnixTimeToDate
    }
    return UnixTimeToDate
}