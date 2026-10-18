[...document.querySelectorAll('.nav-item')].find(e => e.textContent.includes('偏好设置')).click();
setTimeout(() => document.querySelector('.main').scrollTop = 0, 50);
