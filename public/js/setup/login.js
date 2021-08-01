  window.addEventListener('load', () => {
    const url = new URL(location.href);
    const err = url.searchParams.get('err');
    const errorDoc = document.getElementById('login-error');
    if (err == null) return;
    switch (err) {
        case '1':
            errorDoc.getElementsByTagName('span')[0].textContent = 'Invalid login code. Please copy the code from the console.';
            errorDoc.style.display = 'block';
            break;
    }
});
