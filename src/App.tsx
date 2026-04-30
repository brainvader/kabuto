import { OntologyPanel } from "./components/OntologyPanel";
import './App.css';

function App() {
  return (
    <main style={{ background: '#111827', minHeight: '100vh', padding: '24px' }}>
      <OntologyPanel
        onIngestionComplete={() => console.log('完了')}
        onIngestionError={(msg) => console.error(msg)}
      />
    </main>
  );
}

export default App;