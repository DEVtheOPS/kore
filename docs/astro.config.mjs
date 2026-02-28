// @ts-check
import { defineConfig } from 'astro/config'
import starlight from '@astrojs/starlight'

export default defineConfig({
  site: 'https://devtheops.github.io',
  base: '/kore',
  integrations: [
    starlight({
      title: 'Kore',
      description:
        'Kubernetes Orchestration and Resource Explorer — a lightweight, open-source desktop IDE for managing Kubernetes clusters.',
      logo: {
        light: './src/assets/brand/logo-light.svg',
        dark: './src/assets/brand/logo-dark.svg',
        replacesTitle: false,
      },
      social: [
        {
          icon: 'github',
          label: 'GitHub',
          href: 'https://github.com/DEVtheOPS/kore',
        },
      ],
      sidebar: [
        {
          label: 'Getting Started',
          items: [
            { label: 'Installation', slug: 'getting-started/installation' },
            { label: 'Quick Start', slug: 'getting-started/quick-start' },
          ],
        },
        {
          label: 'Features',
          items: [
            { label: 'Cluster Management', slug: 'features/cluster-management' },
            { label: 'Workloads', slug: 'features/workloads' },
            { label: 'Helm', slug: 'features/helm' },
            { label: 'RBAC', slug: 'features/rbac' },
            { label: 'Storage', slug: 'features/storage' },
            { label: 'Networking', slug: 'features/networking' },
          ],
        },
        {
          label: 'Reference',
          items: [
            { label: 'Supported Resources', slug: 'reference/supported-resources' },
            { label: 'Keyboard Shortcuts', slug: 'reference/keyboard-shortcuts' },
          ],
        },
      ],
      customCss: ['./src/styles/custom.css'],
      editLink: {
        baseUrl: 'https://github.com/DEVtheOPS/kore/edit/main/docs/',
      },
    }),
  ],
})
