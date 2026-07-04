import React, { useState } from 'react';
import { Sidebar } from './components/Sidebar';
import { TipTapEditor } from './components/TipTapEditor';
import { AgentPanel } from './components/AgentPanel';
import { TelemetryBar } from './components/TelemetryBar';
import { GatewayView } from './components/GatewayView';
import { TimelineForeshadowView } from './components/TimelineForeshadowView';
import { ExportModal } from './components/ExportModal';
import { ModelSettingsPage } from './pages/ModelSettingsPage';
import { UsageAuditPage } from './pages/UsageAuditPage';
import { AgentWorkflowPage } from './pages/AgentWorkflowPage';

export function App() {
  const [activeTab, setActiveTab] = useState<'editor' | 'gateway' | 'timeline' | 'models' | 'audit' | 'workflow'>('editor');
  const [isExportOpen, setIsExportOpen] = useState(false);

  return (
    <div className="flex flex-col h-screen w-screen bg-slate-950 overflow-hidden font-sans">
      {/* Main Workspace Row */}
      <div className="flex-1 flex overflow-hidden">
        {/* Left Tree Sidebar */}
        <Sidebar
          activeTab={activeTab}
          setActiveTab={setActiveTab}
          onOpenExport={() => setIsExportOpen(true)}
        />

        {/* Center Main View Switcher */}
        {activeTab === 'editor' && <TipTapEditor />}
        {activeTab === 'workflow' && <AgentWorkflowPage />}
        {activeTab === 'gateway' && <GatewayView />}
        {activeTab === 'timeline' && <TimelineForeshadowView />}
        {activeTab === 'models' && <ModelSettingsPage />}
        {activeTab === 'audit' && <UsageAuditPage />}

        {/* Right AI Agent Co-pilot Panel (Visible in editor view) */}
        {activeTab === 'editor' && <AgentPanel />}
      </div>

      {/* Bottom Telemetry Status Bar */}
      <TelemetryBar />

      {/* Novel Multi-Format Export Modal */}
      <ExportModal isOpen={isExportOpen} onClose={() => setIsExportOpen(false)} />
    </div>
  );
}

export default App;
