// Vue Router 配置
// Layout 包裹主页、项目列表、工作区、设置等页面

import { createRouter, createWebHashHistory } from 'vue-router';
import AppLayout from '../views/AppLayout.vue';
import HomeView from '../views/HomeView.vue';
import ProjectListView from '../views/ProjectListView.vue';
import WorkspaceView from '../views/WorkspaceView.vue';
import ProjectHistoryView from '../views/ProjectHistoryView.vue';

export const routes = [
  {
    path: '/',
    component: AppLayout,
    children: [
      {
        path: '',
        name: 'home',
        component: HomeView,
      },
      {
        path: 'projects',
        name: 'projects',
        component: ProjectListView,
      },
      {
        path: 'workspace/:projectId',
        name: 'workspace',
        component: WorkspaceView,
        props: true,
      },
      {
        path: 'history/:projectId',
        name: 'history',
        component: ProjectHistoryView,
        props: true,
      },
    ],
  },
];

export const router = createRouter({
  history: createWebHashHistory(),
  routes,
});
