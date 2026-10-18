import React from 'react';
import { useQuery } from '@tanstack/react-query';
import { apiClient } from '../api/client';
import {
  Clock,
  CheckCircle2,
  AlertTriangle,
  Send,
  Layers,
  Globe2,
  Share2,
  Calendar,
  Lock,
  ArrowUpRight,
} from 'lucide-react';

interface DashboardViewProps {
  onNavigate: (view: string) => void;
}

export const DashboardView: React.FC<DashboardViewProps> = ({ onNavigate }) => {
  const { data: stats, isLoading, refetch } = useQuery({
    queryKey: ['dashboard-stats'],
    queryFn: apiClient.getStats,
    refetchInterval: 15000,
  });

  if (isLoading || !stats) {
    return (
      <div className="flex h-96 items-center justify-center">
        <div className="h-10 w-10 animate-spin rounded-full border-4 border-indigo-500 border-t-transparent" />
      </div>
    );
  }

  const { quota, next_slot, publications, social_accounts, curation, sources } = stats;
  const quotaPercent = Math.min(100, Math.round((quota.published_count / quota.max_limit) * 100));

  // Créneaux stricts imposés par la règle 4 (06h, 09h, 12h, 15h, 18h)
  const cronSlots = ['06:00', '09:00', '12:00', '15:00', '18:00'];

  return (
    <div className="space-y-8 animate-fadeIn">
      {/* En-tête */}
      <div className="flex flex-col md:flex-row md:items-center justify-between gap-4">
        <div>
          <h1 className="text-3xl font-bold tracking-tight text-white flex items-center gap-3">
            <span>Tableau de Bord & Quotas</span>
            <span className="text-xs px-2.5 py-1 rounded-full bg-indigo-500/10 text-indigo-400 border border-indigo-500/20 font-medium">
              Temps réel
            </span>
          </h1>
          <p className="text-slate-400 text-sm mt-1">
            Veille continue, quotas de publication journaliers et santé des canaux
          </p>
        </div>
        <button
          onClick={() => refetch()}
          className="self-start md:self-auto px-4 py-2 bg-slate-800 hover:bg-slate-700 text-slate-200 text-sm font-medium rounded-lg border border-slate-700 transition"
        >
          Actualiser
        </button>
      </div>

      {/* Section Verrou de Quota & Prochain Créneau */}
      <div className="grid grid-cols-1 lg:grid-cols-3 gap-6">
        {/* Quota Journalier (5 max) */}
        <div className="lg:col-span-2 bg-slate-900/80 border border-slate-800 rounded-2xl p-6 shadow-xl relative overflow-hidden">
          <div className="absolute top-0 right-0 p-8 opacity-5">
            <Lock className="w-48 h-48" />
          </div>

          <div className="flex items-center justify-between mb-4">
            <div className="flex items-center gap-3">
              <div className={`p-2.5 rounded-xl ${quota.is_full ? 'bg-amber-500/10 text-amber-400' : 'bg-indigo-500/10 text-indigo-400'}`}>
                {quota.is_full ? <Lock className="w-6 h-6" /> : <Calendar className="w-6 h-6" />}
              </div>
              <div>
                <h3 className="text-lg font-semibold text-white">Verrou de Quota Journalier</h3>
                <p className="text-xs text-slate-400">Règle stricte : 5 publications maximum par jour</p>
              </div>
            </div>
            <span className={`px-3 py-1 rounded-full text-xs font-semibold ${
              quota.is_full
                ? 'bg-amber-500/20 text-amber-300 border border-amber-500/30'
                : 'bg-emerald-500/20 text-emerald-300 border border-emerald-500/30'
            }`}>
              {quota.is_full ? 'Quota atteint (5/5)' : `${quota.remaining} créneaux restants`}
            </span>
          </div>

          {/* Jauge de progression */}
          <div className="space-y-2 my-6">
            <div className="flex justify-between text-sm">
              <span className="text-slate-400 font-medium">Progression du jour ({stats.today})</span>
              <span className="text-white font-bold">{quota.published_count} / {quota.max_limit} publiées</span>
            </div>
            <div className="w-full bg-slate-800 h-3.5 rounded-full overflow-hidden p-0.5 border border-slate-700/50">
              <div
                className={`h-full rounded-full transition-all duration-700 ${
                  quota.is_full
                    ? 'bg-gradient-to-r from-amber-500 to-red-500'
                    : 'bg-gradient-to-r from-indigo-500 to-blue-500'
                }`}
                style={{ width: `${quotaPercent}%` }}
              />
            </div>
          </div>

          {/* Slots Horaires du Cron Job (06h, 09h, 12h, 15h, 18h) */}
          <div className="pt-4 border-t border-slate-800/80">
            <h4 className="text-xs uppercase font-bold tracking-wider text-slate-400 mb-3">
              Créneaux d'exécution programmés (toutes les 3h dès 06h00)
            </h4>
            <div className="grid grid-cols-5 gap-2">
              {cronSlots.map((slot, index) => {
                const isExecuted = index < quota.published_count;
                return (
                  <div
                    key={slot}
                    className={`flex flex-col items-center justify-center p-3 rounded-xl border transition ${
                      isExecuted
                        ? 'bg-emerald-950/40 border-emerald-500/40 text-emerald-400'
                        : 'bg-slate-800/40 border-slate-700/50 text-slate-400'
                    }`}
                  >
                    <span className="text-xs font-semibold">{slot}</span>
                    <span className="text-[10px] mt-1 flex items-center gap-1">
                      {isExecuted ? (
                        <>
                          <CheckCircle2 className="w-3 h-3 text-emerald-400" /> Fait
                        </>
                      ) : (
                        'En attente'
                      )}
                    </span>
                  </div>
                );
              })}
            </div>
          </div>
        </div>

        {/* Prochain Créneau & Statut Scheduler */}
        <div className="bg-slate-900/80 border border-slate-800 rounded-2xl p-6 shadow-xl flex flex-col justify-between">
          <div>
            <div className="flex items-center gap-3 mb-4">
              <div className="p-2.5 rounded-xl bg-blue-500/10 text-blue-400">
                <Clock className="w-6 h-6" />
              </div>
              <div>
                <h3 className="text-lg font-semibold text-white">Prochain Envoi</h3>
                <p className="text-xs text-slate-400">Tokio Cron Scheduler</p>
              </div>
            </div>

            <div className="bg-slate-800/60 rounded-xl p-5 border border-slate-700/50 my-4 text-center">
              <span className="text-xs uppercase tracking-wider text-slate-400 font-semibold">Créneau prévu</span>
              <div className="text-3xl font-extrabold text-white mt-1 text-indigo-400 tracking-tight">
                {next_slot.label}
              </div>
              <p className="text-xs text-slate-400 mt-2">
                Expression : <code className="text-indigo-300 bg-slate-900 px-2 py-0.5 rounded">0 0 6,9,12,15,18 * * *</code>
              </p>
            </div>
          </div>

          <div className="space-y-3">
            <div className="flex items-center justify-between text-xs text-slate-400 p-2.5 rounded-lg bg-slate-950/40 border border-slate-800">
              <span>Publications en file d'attente</span>
              <span className="font-bold text-white px-2 py-0.5 bg-indigo-500/20 text-indigo-300 rounded">
                {publications.pending}
              </span>
            </div>

            <button
              onClick={() => onNavigate('queue')}
              className="w-full flex items-center justify-center gap-2 py-2.5 bg-indigo-600 hover:bg-indigo-500 text-white text-sm font-semibold rounded-xl transition shadow-lg shadow-indigo-600/20"
            >
              <span>Voir la file d'attente</span>
              <ArrowUpRight className="w-4 h-4" />
            </button>
          </div>
        </div>
      </div>

      {/* Métriques Clés */}
      <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-5">
        {/* Curation */}
        <div
          onClick={() => onNavigate('curation')}
          className="bg-slate-900/70 hover:bg-slate-900 border border-slate-800 hover:border-slate-700 p-5 rounded-2xl cursor-pointer transition shadow-md group"
        >
          <div className="flex items-center justify-between">
            <span className="text-xs font-semibold text-slate-400">Flux de Curation</span>
            <div className="p-2 rounded-lg bg-amber-500/10 text-amber-400 group-hover:scale-110 transition">
              <Layers className="w-5 h-5" />
            </div>
          </div>
          <div className="mt-3 flex items-baseline gap-2">
            <span className="text-3xl font-bold text-white">{curation.new}</span>
            <span className="text-xs text-slate-400">à traiter</span>
          </div>
          <div className="mt-2 text-xs text-slate-500 flex justify-between">
            <span>{curation.queued} en file</span>
            <span>{curation.total} au total</span>
          </div>
        </div>

        {/* Publications */}
        <div
          onClick={() => onNavigate('queue')}
          className="bg-slate-900/70 hover:bg-slate-900 border border-slate-800 hover:border-slate-700 p-5 rounded-2xl cursor-pointer transition shadow-md group"
        >
          <div className="flex items-center justify-between">
            <span className="text-xs font-semibold text-slate-400">Diffusions Réussies</span>
            <div className="p-2 rounded-lg bg-emerald-500/10 text-emerald-400 group-hover:scale-110 transition">
              <Send className="w-5 h-5" />
            </div>
          </div>
          <div className="mt-3 flex items-baseline gap-2">
            <span className="text-3xl font-bold text-white">{publications.published}</span>
            <span className="text-xs text-slate-400">posts envoyés</span>
          </div>
          <div className="mt-2 text-xs text-slate-500 flex justify-between">
            <span>{publications.failed} en échec</span>
            <span>{publications.pending} en attente</span>
          </div>
        </div>

        {/* Sources de Scraping */}
        <div
          onClick={() => onNavigate('sources')}
          className="bg-slate-900/70 hover:bg-slate-900 border border-slate-800 hover:border-slate-700 p-5 rounded-2xl cursor-pointer transition shadow-md group"
        >
          <div className="flex items-center justify-between">
            <span className="text-xs font-semibold text-slate-400">Sources de Veille</span>
            <div className="p-2 rounded-lg bg-blue-500/10 text-blue-400 group-hover:scale-110 transition">
              <Globe2 className="w-5 h-5" />
            </div>
          </div>
          <div className="mt-3 flex items-baseline gap-2">
            <span className="text-3xl font-bold text-white">{sources.active}</span>
            <span className="text-xs text-slate-400">sources actives</span>
          </div>
          <div className="mt-2 text-xs text-slate-500 flex justify-between">
            <span>Total : {sources.total}</span>
            <span className="text-indigo-400 font-medium">Scraping prêt</span>
          </div>
        </div>

        {/* Comptes Sociaux */}
        <div
          onClick={() => onNavigate('social')}
          className="bg-slate-900/70 hover:bg-slate-900 border border-slate-800 hover:border-slate-700 p-5 rounded-2xl cursor-pointer transition shadow-md group"
        >
          <div className="flex items-center justify-between">
            <span className="text-xs font-semibold text-slate-400">Santé Réseaux Sociaux</span>
            <div className={`p-2 rounded-lg ${social_accounts.errors > 0 ? 'bg-red-500/10 text-red-400' : 'bg-purple-500/10 text-purple-400'} group-hover:scale-110 transition`}>
              <Share2 className="w-5 h-5" />
            </div>
          </div>
          <div className="mt-3 flex items-baseline gap-2">
            <span className="text-3xl font-bold text-white">{social_accounts.healthy} / {social_accounts.active}</span>
            <span className="text-xs text-slate-400">actifs & sains</span>
          </div>
          <div className="mt-2 text-xs text-slate-500 flex justify-between">
            <span className={social_accounts.errors > 0 ? 'text-red-400 font-medium' : ''}>
              {social_accounts.errors} en alerte
            </span>
            <span>AES-256 Chiffré</span>
          </div>
        </div>
      </div>

      {/* Historique des envois du jour */}
      <div className="bg-slate-900/80 border border-slate-800 rounded-2xl p-6 shadow-xl">
        <h3 className="text-lg font-semibold text-white mb-4 flex items-center gap-2">
          <span>Historique des Diffusions du Jour</span>
          <span className="text-xs text-slate-500 font-normal">({quota.history.length} exécutions enregistrées)</span>
        </h3>

        {quota.history.length === 0 ? (
          <div className="text-center py-8 text-slate-500 text-sm">
            Aucune diffusion n'a encore été effectuée aujourd'hui ({stats.today}).
          </div>
        ) : (
          <div className="divide-y divide-slate-800">
            {quota.history.map((entry, idx) => (
              <div key={idx} className="py-3 flex items-center justify-between text-sm">
                <div className="flex items-center gap-3">
                  <div className="h-2 w-2 rounded-full bg-emerald-400" />
                  <span className="font-medium text-slate-200">Publication #{entry.publication_id.slice(-6)}</span>
                  <span className="text-xs px-2 py-0.5 rounded bg-slate-800 text-indigo-300 font-mono">
                    {entry.slot}
                  </span>
                </div>
                <span className="text-xs text-slate-400">
                  {new Date(entry.executed_at).toLocaleTimeString('fr-FR')}
                </span>
              </div>
            ))}
          </div>
        )}
      </div>
    </div>
  );
};
