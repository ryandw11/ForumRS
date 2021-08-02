window.addEventListener('load', () => {
  const url = new URL(location.href);
  const err = url.searchParams.get('err');
  const errorDoc = document.getElementById('general-error');
  if (err == null) return;
  switch (err) {
    case '1':
      errorDoc.getElementsByTagName('span')[0].textContent = 'A website name is required.';
      errorDoc.style.display = 'block';
      break;
    case '2':
      errorDoc.getElementsByTagName('span')[0].textContent = 'Invalid IP Address. If you are not sure what to put there, then just use 127.0.0.1!';
      errorDoc.style.display = 'block';
      break;
    case '3':
      errorDoc.getElementsByTagName('span')[0].textContent = 'Invalid Port. If you are not sure what to put there, then just use 8080!';
      errorDoc.style.display = 'block';
      break;
    case '4':
      errorDoc.getElementsByTagName('span')[0].textContent = 'Invalid Domain. Please tell ForumRS what domain you intened on using. Do not include http or https. Example: forumrs.example.com.';
      errorDoc.style.display = 'block';
      break;
  }
});
