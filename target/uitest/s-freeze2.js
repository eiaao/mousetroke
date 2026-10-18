[...document.querySelectorAll('.guide .btn.primary')][0].click();
setTimeout(() => document.getAnimations().forEach((a) => { a.pause(); a.currentTime = 2000; }), 100);
