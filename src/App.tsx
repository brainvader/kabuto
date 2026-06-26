import { StockSearch } from './components/StockSearch'
import { PlayGround } from './components/PlayGround'
import { useKabutoStore } from './store/useKabutoStore'

function App() {
  const setSelection = useKabutoStore((s) => s.setSelection)

  return (
    <div
      style={{
        display: 'grid',
        gridTemplateColumns: '300px 1fr',
        gridTemplateRows: '1fr 240px',
        height: '100vh',
        background: '#09090b',
        overflow: 'hidden',
      }}
    >
      {/* CTX-1: Stock Search */}
      <div
        data-testid="stock-search-panel"
        style={{ gridRow: '1 / 3', borderRight: '1px solid #1e2333', overflow: 'hidden' }}
      >
        <StockSearch onSelect={setSelection} />
      </div>

      {/* PlayGround */}
      <div
        data-testid="playground-panel"
        style={{ borderBottom: '1px solid #1e2333', overflow: 'hidden' }}
      >
        <PlayGround />
      </div>

      {/* CTX-3: Metrics */}
      <div
        data-testid="metrics-panel"
        style={{ overflow: 'hidden' }}
      />
    </div>
  )
}

export default App