import type { Preview } from '@storybook/react-vite'
import '../src/App.css'

const preview: Preview = {
  parameters: {
    controls: {
      matchers: {
        color: /(background|color)$/i,
        date: /Date$/i,
      },
    },
    a11y: {
      test: 'todo'
    },
    backgrounds: {
      default: 'kabuto',
      values: [
        { name: 'kabuto', value: '#09090b' },
      ],
    },
  },
  decorators: [
    (Story) => (
      <div className="dark" style={{ background: '#09090b', minHeight: '100vh' }}>
        <Story />
      </div>
    ),
  ],
}

export default preview