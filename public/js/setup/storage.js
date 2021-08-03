const sqliteConfig = document.getElementById("sql-config");
const mysqlConfig = document.getElementById("mysql-config");
const postgreConfig = document.getElementById("postgre-config");

mysqlConfig.style.display = "none";
postgreConfig.style.display = "none";

for (let elem of mysqlConfig.getElementsByTagName("input")) {
  elem.disabled = true;
}

for (let elem of postgreConfig.getElementsByTagName("input")) {
  elem.disabled = true;
}

let currentActiveConfig = sqliteConfig;

document.getElementById("dbType").addEventListener('change', evt => {
  for (let elem of currentActiveConfig.getElementsByTagName("input")) {
    elem.disabled = true;
  }

  currentActiveConfig.style.display = "none";

  switch (evt.target.value) {
    case "SQLite":
      currentActiveConfig = sqliteConfig;
      break;
    case "MySQL":
      currentActiveConfig = mysqlConfig;
      break;
    case "PostgreSQL":
      currentActiveConfig = postgreConfig;
      break;
  }

  for (let elem of currentActiveConfig.getElementsByTagName("input")) {
    elem.disabled = false;
  }
  currentActiveConfig.style.display = "block";
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
