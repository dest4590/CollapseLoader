const fs = require('fs');
fetch('https://t.me/s/CollapseLoaderReborn', {headers: {'User-Agent': 'Mozilla/5.0 (Windows NT 10.0; Win64; x64)'}})
  .then(r => r.text())
  .then(t => {
    fs.writeFileSync('telegram_test.html', t);
    console.log('Saved to telegram_test.html');
  });
