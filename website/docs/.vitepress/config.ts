import { defineConfig } from 'vitepress'

export default defineConfig({
  title: 'roea-ai',
  description: 'Observability for AI Coding Agents',

  head: [
    ['link', { rel: 'icon', type: 'image/svg+xml', href: '/logo.svg' }],
    ['meta', { name: 'theme-color', content: '#3b82f6' }],
    ['meta', { property: 'og:type', content: 'website' }],
    ['meta', { property: 'og:title', content: 'roea-ai Documentation' }],
    ['meta', { property: 'og:description', content: 'See what your AI coding agents are really doing' }],
    ['meta', { property: 'og:url', content: 'https://roea.ai' }],
  ],

  themeConfig: {
    logo: '/logo.svg',

    nav: [
      { text: 'Guide', link: '/guide/introduction' },
      { text: 'Features', link: '/features/process-monitoring' },
      { text: 'Reference', link: '/reference/configuration' },
      { text: 'Download', link: '/download' },
      {
        text: 'Resources',
        items: [
          { text: 'GitHub', link: 'https://github.com/your-org/roea-ai' },
          { text: 'Releases', link: 'https://github.com/your-org/roea-ai/releases' },
          { text: 'Contributing', link: '/contributing' }
        ]
      }
    ],

    sidebar: {
      '/guide/': [
        {
          text: 'Getting Started',
          items: [
            { text: 'Introduction', link: '/guide/introduction' },
            { text: 'Installation', link: '/guide/installation' },
            { text: 'Quick Start', link: '/guide/quick-start' },
            { text: 'System Requirements', link: '/guide/requirements' }
          ]
        },
        {
          text: 'Core Concepts',
          items: [
            { text: 'How It Works', link: '/guide/how-it-works' },
            { text: 'Agent Detection', link: '/guide/agent-detection' },
            { text: 'Data Storage', link: '/guide/storage' }
          ]
        }
      ],
      '/features/': [
        {
          text: 'Monitoring Features',
          items: [
            { text: 'Process Monitoring', link: '/features/process-monitoring' },
            { text: 'Network Tracking', link: '/features/network-tracking' },
            { text: 'File Access', link: '/features/file-access' },
            { text: 'Search & Filtering', link: '/features/search' }
          ]
        },
        {
          text: 'Visualization',
          items: [
            { text: 'Process Tree Graph', link: '/features/process-graph' },
            { text: 'Dashboard', link: '/features/dashboard' }
          ]
        },
        {
          text: 'Supported Agents',
          items: [
            { text: 'Claude Code', link: '/features/agents/claude-code' },
            { text: 'Cursor', link: '/features/agents/cursor' },
            { text: 'VS Code Copilot', link: '/features/agents/copilot' },
            { text: 'Windsurf', link: '/features/agents/windsurf' },
            { text: 'Aider', link: '/features/agents/aider' }
          ]
        }
      ],
      '/reference/': [
        {
          text: 'Configuration',
          items: [
            { text: 'Agent Signatures', link: '/reference/configuration' },
            { text: 'Storage Settings', link: '/reference/storage' },
            { text: 'Environment Variables', link: '/reference/environment' }
          ]
        },
        {
          text: 'Advanced',
          items: [
            { text: 'Linux eBPF Setup', link: '/reference/ebpf' },
            { text: 'osquery Integration', link: '/reference/osquery' },
            { text: 'OpenTelemetry Export', link: '/reference/opentelemetry' }
          ]
        },
        {
          text: 'Troubleshooting',
          items: [
            { text: 'Common Issues', link: '/reference/troubleshooting' },
            { text: 'FAQ', link: '/reference/faq' }
          ]
        }
      ]
    },

    socialLinks: [
      { icon: 'github', link: 'https://github.com/your-org/roea-ai' },
      { icon: 'twitter', link: 'https://twitter.com/roea_ai' }
    ],

    footer: {
      message: 'Released under the MIT License.',
      copyright: 'Copyright 2024-present roea-ai contributors'
    },

    search: {
      provider: 'local'
    },

    editLink: {
      pattern: 'https://github.com/your-org/roea-ai/edit/main/website/docs/:path',
      text: 'Edit this page on GitHub'
    },

    lastUpdated: {
      text: 'Last updated',
      formatOptions: {
        dateStyle: 'medium'
      }
    }
  },

  markdown: {
    theme: {
      light: 'github-light',
      dark: 'github-dark'
    },
    lineNumbers: true
  },

  sitemap: {
    hostname: 'https://roea.ai'
  }
})
