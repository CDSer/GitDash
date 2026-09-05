import { createApp } from 'vue';
import { createPinia } from 'pinia';
import './assets/tailwind.css';
import App from './App.vue';

const app = createApp(App);

app.use(createPinia());

// 禁用浏览器默认的右键菜单，桌面端统一使用自定义菜单
document.addEventListener('contextmenu', (event) => {
  event.preventDefault();
});

app.mount('#app');
