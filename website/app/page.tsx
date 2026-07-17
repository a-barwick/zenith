const leadershipOutcomes = [
  {
    label: "Defect timing",
    value: "Before deploy",
    description:
      "Move ordinary type, query-shape, security, and resource mistakes into the local feedback loop.",
  },
  {
    label: "Deployment model",
    value: "Apex-native",
    description:
      "Emit readable, deterministic Apex that fits existing Salesforce delivery and review practices.",
  },
  {
    label: "Adoption path",
    value: "Incremental",
    description:
      "Introduce Zenith module by module beside existing Apex without replacing the production runtime.",
  },
];

const safetyContracts = [
  {
    number: "01",
    milestone: "M4",
    title: "Safer values",
    description:
      "Non-null types, immutable bindings, typed Salesforce IDs, and exhaustive results make domain mistakes explicit.",
    proof: "Nullability · Id<Account> · typed record states",
  },
  {
    number: "02",
    milestone: "M5",
    title: "Query-shaped data",
    description:
      "The type system tracks selected fields and relationship nullability so data access matches the SOQL projection.",
    proof: "Schema-aware queries · selected-field checks",
  },
  {
    number: "03",
    milestone: "M6",
    title: "Governor effects",
    description:
      "SOQL, DML, callout, enqueue, and privilege effects propagate through the call graph and across loops.",
    proof: "Resource contracts · bulk amplification diagnostics",
  },
];

const roadmap = [
  {
    milestone: "M0",
    state: "Complete",
    title: "Compiler foundation",
    description:
      "Rust crate, source identities, file-aware spans, phase-owned diagnostics, and a tested bootstrap CLI.",
  },
  {
    milestone: "M1",
    state: "Active",
    title: "Lexical core",
    description:
      "Apex-compatible tokens, case-insensitive names, source diagnostics, and the first source-to-token slice.",
  },
  {
    milestone: "M3",
    state: "Target",
    title: "Deployable Apex baseline",
    description:
      "The first complete path from checked multi-file Zenith source to deterministic SFDX-compatible Apex.",
  },
  {
    milestone: "M6",
    state: "North star",
    title: "Bulk and governor safety",
    description:
      "The central safety checkpoint: actionable resource diagnostics across calls, loops, and trigger batches.",
  },
];

export default function Home() {
  return (
    <main>
      <nav className="site-nav" aria-label="Primary navigation">
        <a className="wordmark" href="#top" aria-label="Zenith home">
          <span className="wordmark-mark" aria-hidden="true">
            ZN
          </span>
          <span>Zenith</span>
        </a>
        <div className="nav-links">
          <a href="#case">Why Zenith</a>
          <a href="#guarantees">Guarantees</a>
          <a href="#roadmap">Roadmap</a>
        </div>
        <a
          className="nav-cta"
          href="https://github.com/a-barwick/zenith"
          target="_blank"
          rel="noreferrer"
        >
          View repository
          <span aria-hidden="true">↗</span>
        </a>
      </nav>

      <section className="hero" id="top">
        <div className="hero-grid" aria-hidden="true" />
        <div className="hero-copy">
          <div className="eyebrow">
            <span className="status-dot" aria-hidden="true" />
            M1 active · Lexical core
          </div>
          <h1>
            Make Salesforce risk fail at{" "}
            <em>compile time.</em>
          </h1>
          <p className="hero-lede">
            Zenith is a safe, bulk-first language for Salesforce teams. It is
            designed to catch type, query-shape, security-context, and
            governor-limit mistakes locally—then emit readable, deployable
            Apex.
          </p>
          <div className="hero-actions">
            <a className="button button-primary" href="#case">
              Review the engineering case
              <span aria-hidden="true">↓</span>
            </a>
            <a
              className="button button-secondary"
              href="https://github.com/a-barwick/zenith/blob/main/ROADMAP.md"
              target="_blank"
              rel="noreferrer"
            >
              Read the roadmap
              <span aria-hidden="true">↗</span>
            </a>
          </div>
        </div>

        <div className="hero-visual" aria-label="Zenith compiler status">
          <div className="proof-label">Compiler proof / Zenith</div>
          <div className="terminal">
            <div className="terminal-bar">
              <div className="terminal-dots" aria-hidden="true">
                <span />
                <span />
                <span />
              </div>
              <span>source-to-token / M1</span>
              <span className="terminal-live">active</span>
            </div>
            <div className="terminal-body">
              <div className="terminal-command">
                <span className="prompt">$</span>
                <span>zenith tokens examples/hello.zen</span>
              </div>
              <div className="terminal-rule" />
              <div className="terminal-line">
                <span className="pass">READY</span>
                <span>file-aware source spans</span>
              </div>
              <div className="terminal-line">
                <span className="pass">READY</span>
                <span>phase-owned diagnostics</span>
              </div>
              <div className="terminal-line terminal-line-active">
                <span>ACTIVE</span>
                <span>case-insensitive lexical core</span>
              </div>
              <div className="target-block">
                <span>First deployable checkpoint</span>
                <strong>M3 · readable Apex</strong>
              </div>
            </div>
          </div>
          <div className="visual-note">
            <span>Product contract</span>
            <strong>Unsupported behavior fails explicitly.</strong>
          </div>
        </div>
      </section>

      <section className="signal-strip" aria-label="Language principles">
        <div>
          <span>01</span>
          <strong>Bulk-first</strong>
        </div>
        <div>
          <span>02</span>
          <strong>Compile-to-Apex</strong>
        </div>
        <div>
          <span>03</span>
          <strong>Deterministic</strong>
        </div>
        <div>
          <span>04</span>
          <strong>Explicit</strong>
        </div>
      </section>

      <section className="leadership-case section" id="case">
        <div className="section-intro">
          <p className="section-kicker">The leadership case</p>
          <h2>
            The compiler becomes
            <br />
            <span>a governance layer.</span>
          </h2>
          <p>
            Salesforce defects are often platform-shaped rather than
            syntax-shaped. Zenith turns the most expensive recurring failure
            modes into checked language contracts—without asking the
            organization to abandon Apex or Salesforce.
          </p>
        </div>

        <div className="outcomes-grid">
          {leadershipOutcomes.map((outcome) => (
            <article className="outcome-card" key={outcome.label}>
              <p>{outcome.label}</p>
              <h3>{outcome.value}</h3>
              <span>{outcome.description}</span>
            </article>
          ))}
        </div>

        <div className="operating-model">
          <div className="model-label">Target operating model</div>
          <div className="model-flow" role="list">
            <div className="model-step" role="listitem">
              <span>01</span>
              <strong>Author</strong>
              <p>Bulk-first Zenith source</p>
            </div>
            <div className="flow-arrow" aria-hidden="true">
              →
            </div>
            <div className="model-step model-step-highlight" role="listitem">
              <span>02</span>
              <strong>Check + lower</strong>
              <p>Local compiler and ordinary CI</p>
            </div>
            <div className="flow-arrow" aria-hidden="true">
              →
            </div>
            <div className="model-step" role="listitem">
              <span>03</span>
              <strong>Verify + deploy</strong>
              <p>Readable Apex on Salesforce</p>
            </div>
          </div>
        </div>
      </section>

      <section className="guarantees-section section" id="guarantees">
        <div className="guarantees-heading">
          <div>
            <p className="section-kicker section-kicker-dark">
              The language advantage
            </p>
            <h2>Safety that survives compilation.</h2>
          </div>
          <p>
            Each Zenith feature is complete only when its Apex lowering is
            defined, inspectable, source-mapped, and tested. Planned guarantees
            arrive as vertical language slices—not disconnected syntax.
          </p>
        </div>

        <div className="contracts">
          {safetyContracts.map((contract) => (
            <article className="contract" key={contract.number}>
              <div className="contract-meta">
                <span>{contract.number}</span>
                <span>{contract.milestone}</span>
              </div>
              <div>
                <h3>{contract.title}</h3>
                <p>{contract.description}</p>
                <span className="contract-proof">{contract.proof}</span>
              </div>
            </article>
          ))}
        </div>

        <div className="honesty-panel">
          <div className="honesty-title">
            <span>Compatibility posture</span>
            <strong>Measured, never implied.</strong>
          </div>
          <p>
            Salesforce remains the final compatibility oracle. Today, Zenith
            has a completed compiler foundation and an active lexical milestone;
            it does not yet accept or emit Apex-shaped source. The project
            publishes that boundary plainly.
          </p>
          <a
            href="https://github.com/a-barwick/zenith/blob/main/docs/COMPATIBILITY.md"
            target="_blank"
            rel="noreferrer"
          >
            Inspect the compatibility contract
            <span aria-hidden="true">↗</span>
          </a>
        </div>
      </section>

      <section className="roadmap-section section" id="roadmap">
        <div className="roadmap-heading">
          <div>
            <p className="section-kicker">Execution path</p>
            <h2>A deliberate path to enterprise leverage.</h2>
          </div>
          <p>
            The roadmap works backward from deployable, Salesforce-aware
            safety. Each milestone ends in an executable capability with an
            explicit compatibility claim.
          </p>
        </div>

        <div className="roadmap-list">
          {roadmap.map((item) => (
            <article className="roadmap-item" key={item.milestone}>
              <div className="roadmap-marker">
                <span>{item.milestone}</span>
                <i aria-hidden="true" />
              </div>
              <div className="roadmap-copy">
                <div className="roadmap-title">
                  <h3>{item.title}</h3>
                  <span className={`state state-${item.state.toLowerCase().replace(" ", "-")}`}>
                    {item.state}
                  </span>
                </div>
                <p>{item.description}</p>
              </div>
            </article>
          ))}
        </div>
      </section>

      <section className="closing-section">
        <div className="closing-kicker">The engineering bet</div>
        <h2>
          Shift platform risk left.
          <br />
          <em>Keep the platform.</em>
        </h2>
        <p>
          Zenith is building toward a safer Salesforce development model:
          stronger local guarantees, transparent generated Apex, and targeted
          final verification where the org is uniquely authoritative.
        </p>
        <div className="closing-actions">
          <a
            className="button button-light"
            href="https://github.com/a-barwick/zenith"
            target="_blank"
            rel="noreferrer"
          >
            Follow the build
            <span aria-hidden="true">↗</span>
          </a>
          <a
            className="text-link"
            href="https://github.com/a-barwick/zenith/blob/main/docs/VISION.md"
            target="_blank"
            rel="noreferrer"
          >
            Read the vision
            <span aria-hidden="true">↗</span>
          </a>
        </div>
      </section>

      <footer>
        <a className="wordmark footer-wordmark" href="#top">
          <span className="wordmark-mark" aria-hidden="true">
            ZN
          </span>
          <span>Zenith</span>
        </a>
        <p>Safe, bulk-first Salesforce development.</p>
        <div className="footer-links">
          <a
            href="https://github.com/a-barwick/zenith"
            target="_blank"
            rel="noreferrer"
          >
            GitHub
          </a>
          <a
            href="https://github.com/a-barwick/zenith/blob/main/ROADMAP.md"
            target="_blank"
            rel="noreferrer"
          >
            Roadmap
          </a>
          <a
            href="https://github.com/a-barwick/zenith/blob/main/docs/COMPATIBILITY.md"
            target="_blank"
            rel="noreferrer"
          >
            Compatibility
          </a>
        </div>
      </footer>
    </main>
  );
}
