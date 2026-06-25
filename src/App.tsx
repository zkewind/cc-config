import { AppProviders } from "./AppProviders";
import { AppRouter } from "./AppRouter";
import { AppLayout } from "./AppLayout";

function App() {
  return (
    <AppProviders>
      <AppRouter>
        <AppLayout />
      </AppRouter>
    </AppProviders>
  );
}

export default App;
