import { createApp } from 'vue';
import { createPinia } from 'pinia';
import './assets/tailwind.css';
import './components/ui/col-resize.css';
import App from './App.vue';
import { router } from './router';

const app = createApp(App);

app.use(createPinia());
app.use(router);

// 禁用浏览器默认的右键菜单，桌面端统一使用自定义菜单
document.addEventListener('contextmenu', (event) => {
  event.preventDefault();
});

app.mount('#app');
