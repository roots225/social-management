import React, { useState } from 'react';
import {
  LayoutDashboard,
  Layers,
  Send,
  Globe2,
  Share2,
  Radio,
  Github,
  Bell,
  Sparkles,
} from 'lucide-react';
import { DashboardView } from './views/DashboardView';
import { CurationView } from './views/CurationView';
import { QueueView } from './views/QueueView';
import { SourcesView } from './views/SourcesView';
import { SocialAccountsView } from './views/SocialAccountsView';

export const App: React.FC = () => {
  const [activeTab, setActiveTab] = useState<'dashboard' | 'curation' | 'queue' | 'sources' | 'social'>('dashboard');

  const navItems = [
    { id: 'dashboard', label: 'Dashboard & Quotas', icon: LayoutDashboard },
    { id: 'curation', label: 'Flux de Curation', icon: Layers },
    { id: 'queue', label: 'File de Publication', icon: Send },
    { id: 'sources', label: 'Sources de Veille', icon: Globe2 },
    { id: 'social', label: 'Comptes Sociaux', icon: Share2 },
  ];

  return (
    <div className="flex min-h-screen bg-slate-950 text-slate-100">
      {/* Sidebar Navigation */}
      <aside className="w-64 bg-slate-900/90 border-r border-slate-800 flex flex-col justify-between p-5 fixed h-full z-20">
        <div className="space-y-6">
          {/* Logo Brand */}
          <div className="flex items-center gap-3 px-2">
            <div className="h-10 w-10 rounded-xl bg-gradient-to-tr from-indigo-600 to-purple-500 flex items-center justify-center shadow-lg shadow-indigo-600/30">
              <Radio className="w-5 h-5 text-white" />
            </div>
            <div>
              <span className="font-extrabold text-base tracking-tight text-white block">
                Social Pulse
              </span>
              <span className="text-[10px] text-indigo-400 font-mono tracking-wide uppercase">
                Rust • Axum • Mongo
              </span>
            </div>
          </div>

          {/* Navigation Links */}
          <nav className="space-y-1.5 pt-4">
            {navItems.map((item) => {
              const Icon = item.icon;
              const isActive = activeTab === item.id;
              return (
                <button
                  key={item.id}
                  onClick={() => setActiveTab(item.id as any)}
                  className={`w-full flex items-center gap-3 px-3.5 py-2.5 rounded-xl text-xs font-semibold transition ${
                    isActive
                      ? 'bg-indigo-600 text-white shadow-lg shadow-indigo-600/20'
                      : 'text-slate-400 hover:text-slate-200 hover:bg-slate-800/60'
                  }`}
                >
                  <Icon className={`w-4 h-4 ${isActive ? 'text-white' : 'text-slate-400'}`} />
                  <span>{item.label}</span>
                </button>
              );
            })}
          </nav>
        </div>

        {/* Footer Sidebar */}
        <div className="pt-4 border-t border-slate-800/80 space-y-3">
          <div className="bg-slate-950 p-3 rounded-xl border border-slate-800 text-xs space-y-1">
            <div className="flex items-center gap-2 text-emerald-400 font-semibold">
              <span className="h-2 w-2 rounded-full bg-emerald-400 animate-pulse" />
              <span>Backend Connecté</span>
            </div>
            <p className="text-[11px] text-slate-500">
              Docker compose orchestré sur le port 8080
            </p>
          </div>

          <div className="flex items-center justify-between text-xs text-slate-500 px-1">
            <span>v0.1.0 • Monorepo</span>
            <span className="text-[10px] font-mono">AES-256-GCM</span>
          </div>
        </div>
      </aside>

      {/* Main Content Area */}
      <div className="flex-1 ml-64 flex flex-col min-h-screen">
        {/* Top Header */}
        <header className="h-16 border-b border-slate-800/80 bg-slate-900/40 backdrop-blur-md px-8 flex items-center justify-between sticky top-0 z-10">
          <div className="flex items-center gap-2 text-xs text-slate-400">
            <span>Plateforme Automatisée</span>
            <span>/</span>
            <span className="text-white font-medium capitalize">
              {navItems.find((n) => n.id === activeTab)?.label}
            </span>
          </div>

          <div className="flex items-center gap-4">
            <div className="flex items-center gap-2 px-3 py-1 bg-slate-800/80 rounded-full border border-slate-700/60 text-xs">
              <span className="h-2 w-2 rounded-full bg-indigo-400" />
              <span className="text-slate-300">Cron actif : 5 créneaux / jour</span>
            </div>
          </div>
        </header>

        {/* Dynamic View Content */}
        <main className="flex-1 p-8 overflow-y-auto max-w-7xl w-full">
          {activeTab === 'dashboard' && <DashboardView onNavigate={(view) => setActiveTab(view as any)} />}
          {activeTab === 'curation' && <CurationView />}
          {activeTab === 'queue' && <QueueView />}
          {activeTab === 'sources' && <SourcesView />}
          {activeTab === 'social' && <SocialAccountsView />}
        </main>
      </div>
    </div>
  );
};
