document.getElementById("useSSL").addEventListener('change', evt => {
  document.getElementById("privateKey").disabled = !evt.target.checked;
  document.getElementById("publicKey").disabled = !evt.target.checked;
});

document.getElementById("useCaptch").addEventListener('change', evt => {
  document.getElementById("siteKey").disabled = !evt.target.checked;
  document.getElementById("secretKey").disabled = !evt.target.checked;
});

window.addEventListener('load', () => {
  const url = new URL(location.href);
  const err = url.searchParams.get('err');
  const errorDoc = document.getElementById('ssl-error');
  if (err == null) return;
  switch (err) {
    case '1':
      errorDoc.getElementsByTagName('span')[0].textContent = 'Please specify a public and private key for SSL.';
      errorDoc.style.display = 'block';
      break;
    case '2':
      errorDoc.getElementsByTagName('span')[0].textContent = 'Please enter a valid private key that is a pem or asc1 file.';
      errorDoc.style.display = 'block';
      break;
    case '3':
      errorDoc.getElementsByTagName('span')[0].textContent = 'Please enter a valid public key that is a pem or asc1 file.';
      errorDoc.style.display = 'block';
      break;
    case '4':
      errorDoc.getElementsByTagName('span')[0].textContent = 'The server cannot find the specified private key. Does the file exist and ForumRS has sufficent permissions to access it?';
      errorDoc.style.display = 'block';
      break;
    case '5':
      errorDoc.getElementsByTagName('span')[0].textContent = 'The server cannot find the specified public key. Does the file exist and ForumRS has sufficent permissions to access it?';
      errorDoc.style.display = 'block';
      break;
  }
});

window.addEventListener('load', () => {
  const url = new URL(location.href);
  const err = url.searchParams.get('err');
  const errorDoc = document.getElementById('captcha-error');
  if (err == null) return;
  switch (err) {
    case '6':
      errorDoc.getElementsByTagName('span')[0].textContent = 'Please specify a Site and Secret key.';
      errorDoc.style.display = 'block';
      break;
    case '7':
      errorDoc.getElementsByTagName('span')[0].textContent = 'Please specify a valid Site and Secret key.';
      errorDoc.style.display = 'block';
      break;
  }
});
