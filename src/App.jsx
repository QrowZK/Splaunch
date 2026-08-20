import React from "react";
import SplaunchScreen from "./screens/SplaunchScreen.jsx";
import ErrorBoundary from "./ErrorBoundary.jsx";
import { inTauri } from "./net/connection.ts";
import { maps as loadMaps, launchPreview, testScenario } from "./net/splaunch.ts";

/* Splaunch is one screen, so this is the whole application: find the engine,
   fetch the map catalogue, and hand both to the editor.
 *
 * It is deliberately not a lobby. There is no account, no server connection and
 * nothing to log in to - the only thing it talks to is Zero-K's public content
 * service, for the list of maps. */

export default function App() {
  const [maps, setMaps] = React.useState([]);
  const [mapsError, setMapsError] = React.useState(undefined);
  const [engine, setEngine] = React.useState("");
  const [installError, setInstallError] = React.useState(undefined);
  const [testError, setTestError] = React.useState(undefined);
  const [running, setRunning] = React.useState(false);

  React.useEffect(() => {
    let live = true;
    loadMaps().then(
      m => { if (live) setMaps(m.map(x => x.name)); },
      e => { if (live) setMapsError(String(e?.message ?? e)); },
    );
    return () => { live = false; };
  }, []);

  /* The engine version is not ours to choose: whatever the installation has is
     what a scenario has to run on, so it is discovered rather than configured. */
  React.useEffect(() => {
    let live = true;
    launchPreview().then(
      p => { if (live) setEngine(p.engine || ""); },
      e => { if (live) setInstallError(String(e?.message ?? e)); },
    );
    return () => { live = false; };
  }, []);

  return (
    <div style={{ display: "flex", flexDirection: "column", height: "100%",
      minHeight: 0, background: "var(--surface-base)", overflow: "hidden" }}>
      <ErrorBoundary>
        <SplaunchScreen
          maps={maps}
          engine={engine}
          game=""
          player="Player"
          running={running}
          testError={testError || installError || mapsError}
          onTest={sc => {
            setTestError(undefined);
            setRunning(true);
            testScenario(sc, "Player", engine)
              .catch(e => { setTestError(String(e?.message ?? e)); setRunning(false); });
          }}
        />
      </ErrorBoundary>
      {!inTauri() && (
        <div style={{ padding: "var(--sp-4) var(--sp-6)", borderTop: "1px solid var(--w-12)",
          font: "var(--text-ui-sm)", color: "var(--text-faint)" }}>
          Running in a browser, so nothing can be launched. Open the desktop app.
        </div>
      )}
    </div>
  );
}
