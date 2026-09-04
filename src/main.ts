import { createApp } from 'vue';
import { createPinia } from 'pinia';
// Element Plus 按需引入：组件与组件样式由 unplugin-vue-components 自动注入
// 这里只需引入基础变量样式与深色主题变量
import 'element-plus/theme-chalk/base.css';
import 'element-plus/theme-chalk/dark/css-vars.css';
// 以函数方式调用的组件（ElMessage / ElMessageBox / ElNotification / ElLoading）
// 不经过模板，插件无法自动导入，需要手动引入样式
import 'element-plus/es/components/message/style/css';
import 'element-plus/es/components/message-box/style/css';
import './assets/tailwind.css';
import App from './App.vue';

const app = createApp(App);

app.use(createPinia());

// 禁用浏览器默认的右键菜单，桌面端统一使用自定义菜单
document.addEventListener('contextmenu', (event) => {
  event.preventDefault();
});

app.mount('#app');
