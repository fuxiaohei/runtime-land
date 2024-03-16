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

    window.addEventListener('DOMContentLoaded', () => {
        showActiveTheme(getPreferredTheme())
        document.getElementById('bd-theme').addEventListener('click', () => {
            const storedTheme = getStoredTheme()
            const currentTheme = storedTheme || getPreferredTheme()
            const newTheme = currentTheme === 'light' ? 'dark' : 'light'
            setStoredTheme(newTheme)
            setTheme(newTheme)
            showActiveTheme(newTheme, true)
        });
    })
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

