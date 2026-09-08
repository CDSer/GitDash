// Vue Router 配置
// Layout 包裹项目列表、工作区、设置等页面

import { createRouter, createWebHashHistory } from 'vue-router';
import AppLayout from '../views/AppLayout.vue';
import ProjectListView from '../views/ProjectListView.vue';
import WorkspaceView from '../views/WorkspaceView.vue';

export const routes = [
  {
    path: '/',
    component: AppLayout,
    children: [
      {
        path: '',
        name: 'projects',
        component: ProjectListView,
      },
      {
        path: 'workspace/:projectId',
        name: 'workspace',
        component: WorkspaceView,
        props: true,
      },
    ],
  },
];

export const router = createRouter({
  history: createWebHashHistory(),
  routes,
});
