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

/**
 * Set the active configuration to the one desired.
 * @param {*} desired The desired active configuration.
 */
function swapDBSettings(desired) {
  for (let elem of currentActiveConfig.getElementsByTagName("input")) {
    elem.disabled = true;
  }
  currentActiveConfig.style.display = "none";
  currentActiveConfig = desired;
  for (let elem of currentActiveConfig.getElementsByTagName("input")) {
    elem.disabled = false;
  }
  currentActiveConfig.style.display = "block";

  if (currentActiveConfig == mysqlConfig) {
    document.getElementById("dbType").value = "MySQL";
  } else if (currentActiveConfig == postgreConfig) {
    document.getElementById("dbType").value = "PostgreSQL";
  } else {
    document.getElementById("dbType").value = "SQLite";
  }
}


window.addEventListener('load', () => {
  const url = new URL(location.href);
  const err = url.searchParams.get('err');
  const errorDoc = document.getElementById('db-error');
  if (err == null) return;
  switch (err) {
    case '1':
      errorDoc.getElementsByTagName('span')[0].textContent = 'Please specify a valid database location for SQLite.';
      errorDoc.style.display = 'block';
      swapDBSettings(sqliteConfig);
      break;
    case '2':
      errorDoc.getElementsByTagName('span')[0].textContent = 'Please enter a value for every MySQL box.';
      errorDoc.style.display = 'block';
      swapDBSettings(mysqlConfig);
      break;
    case '3':
      errorDoc.getElementsByTagName('span')[0].textContent = 'Cannot connect to specified MySQL server. Please ensure that the specified MySQL server exists and ForumRS has access to it. More information on the error is specified in the console.';
      errorDoc.style.display = 'block';
      swapDBSettings(mysqlConfig);
      break;
    case '420':
      errorDoc.getElementsByTagName('span')[0].textContent = 'PostgreSQL is not implemented at this time. Please check back later.';
      errorDoc.style.display = 'block';
      swapDBSettings(postgreConfig);
      break;
  }
});
