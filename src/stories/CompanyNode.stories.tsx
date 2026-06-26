import type { Meta, StoryObj } from '@storybook/react-vite'
import {
    ReactFlow,
    ReactFlowProvider,
    Background,
    BackgroundVariant,
    useNodesState,
} from '@xyflow/react'
import { expect, within } from 'storybook/test'

import { nodeTypes } from '@components/nodes/nodeTypes'

const meta: Meta = {
    title: 'Nodes/CompanyNode',
    decorators: [
        (Story) => (
            <div style={{ width: '100vw', height: '100vh', background: '#09090b' }}>
                <ReactFlowProvider>
                    <Story />
                </ReactFlowProvider>
            </div>
        ),
    ],
}
export default meta

type Story = StoryObj

function INPEXFlow() {
    const [nodes, , onNodesChange] = useNodesState([{
        id: 'company-1605',
        type: 'company',
        position: { x: 0, y: 0 },
        data: { code: '1605', name: 'INPEX' },
    }])
    return (
        <ReactFlow
            nodes={nodes}
            edges={[]}
            onNodesChange={onNodesChange}
            nodeTypes={nodeTypes}
            fitView
            colorMode="dark"
            proOptions={{ hideAttribution: true }}
        >
            <Background variant={BackgroundVariant.Dots} color="#1e2333" gap={28} size={0.5} />
        </ReactFlow>
    )
}

function ToyotaFlow() {
    const [nodes, , onNodesChange] = useNodesState([{
        id: 'company-7203',
        type: 'company',
        position: { x: 0, y: 0 },
        data: { code: '7203', name: 'トヨタ自動車' },
    }])
    return (
        <ReactFlow
            nodes={nodes}
            edges={[]}
            onNodesChange={onNodesChange}
            nodeTypes={nodeTypes}
            fitView
            colorMode="dark"
            proOptions={{ hideAttribution: true }}
        >
            <Background variant={BackgroundVariant.Dots} color="#1e2333" gap={28} size={0.5} />
        </ReactFlow>
    )
}

export const INPEX: Story = {
    render: () => <INPEXFlow />,
    play: async ({ canvasElement }) => {
        const canvas = within(canvasElement)
        await expect(await canvas.findByText('1605')).toBeInTheDocument()
        await expect(await canvas.findByText('INPEX')).toBeInTheDocument()
        await expect(await canvas.findByText('company')).toBeInTheDocument()
    },
}

export const Toyota: Story = {
    render: () => <ToyotaFlow />,
    play: async ({ canvasElement }) => {
        const canvas = within(canvasElement)
        await expect(await canvas.findByText('7203')).toBeInTheDocument()
        await expect(await canvas.findByText('トヨタ自動車')).toBeInTheDocument()
    },
}