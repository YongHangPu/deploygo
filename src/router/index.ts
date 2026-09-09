import { createRouter, createMemoryHistory } from 'vue-router'

const router = createRouter({
  history: createMemoryHistory(),
  routes: [
    {
      path: '/',
      name: 'deploy',
      component: () => import('../views/DeployView.vue'),
    },
    {
      path: '/servers',
      name: 'servers',
      component: () => import('../views/ServerConfigView.vue'),
    },
    {
      path: '/history',
      name: 'history',
      component: () => import('../views/HistoryView.vue'),
    },
    {
      path: '/guide',
      name: 'guide',
      component: () => import('../views/GuideView.vue'),
    },
    {
      path: '/activate',
      name: 'activate',
      component: () => import('../views/ActivateView.vue'),
    },
  ],
})

export default router
