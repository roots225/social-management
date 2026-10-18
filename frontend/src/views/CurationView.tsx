import React, { useState } from 'react';
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { apiClient } from '../api/client';
import { CurationStatus, ScrapedItem, SourceCategory } from '../types';
import {
  ExternalLink,
  PlusCircle,
  XCircle,
  Briefcase,
  Newspaper,
  Check,
  MapPin,
  Building,
  User,
  Filter,
  Layers,
} from 'lucide-react';

export const CurationView: React.FC = () => {
  const queryClient = useQueryClient();
  const [statusFilter, setStatusFilter] = useState<CurationStatus>('NEW');
  const [categoryFilter, setCategoryFilter] = useState<string>('ALL');

  // État du modal de mise en file d'attente
  const [selectedItem, setSelectedItem] = useState<ScrapedItem | null>(null);
  const [priority, setPriority] = useState<number>(0);
  const [customText, setCustomText] = useState<string>('');
  const [selectedAccountIds, setSelectedAccountIds] = useState<string[]>([]);

  // Requête des éléments scrapés
  const { data: items = [], isLoading } = useQuery({
    queryKey: ['scraped-items', statusFilter, categoryFilter],
    queryFn: () =>
      apiClient.getScrapedItems({
        status: statusFilter,
        category: categoryFilter !== 'ALL' ? categoryFilter : undefined,
      }),
  });

  // Requête des comptes sociaux pour le modal
  const { data: accounts = [] } = useQuery({
    queryKey: ['social-accounts'],
    queryFn: apiClient.getSocialAccounts,
  });

  // Mutation pour rejeter ou changer le statut
  const updateStatusMutation = useMutation({
    mutationFn: ({ id, status }: { id: string; status: string }) =>
      apiClient.updateItemStatus(id, status),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['scraped-items'] });
      queryClient.invalidateQueries({ queryKey: ['dashboard-stats'] });
    },
  });

  // Mutation pour transformer en publication PENDING
  const queueMutation = useMutation({
    mutationFn: ({
      id,
      priority,
      customText,
      targetAccountIds,
    }: {
      id: string;
      priority: number;
      customText?: string;
      targetAccountIds?: string[];
    }) =>
      apiClient.queueItem(id, {
        priority,
        custom_text: customText || undefined,
        target_account_ids: targetAccountIds?.length ? targetAccountIds : undefined,
      }),
    onSuccess: () => {
      setSelectedItem(null);
      queryClient.invalidateQueries({ queryKey: ['scraped-items'] });
      queryClient.invalidateQueries({ queryKey: ['publications'] });
      queryClient.invalidateQueries({ queryKey: ['dashboard-stats'] });
    },
  });

  const handleOpenQueueModal = (item: ScrapedItem) => {
    setSelectedItem(item);
    setPriority(0);
    // Suggestion de texte automatique
    if (item.metadata?.item_type === 'JobOffer') {
      const co = item.metadata.company ? `chez ${item.metadata.company}` : '';
      const loc = item.metadata.location ? `📍 ${item.metadata.location}` : '';
      setCustomText(`💼 Offre : ${item.title} ${co} ${loc}\n\n👉 Postuler : ${item.original_url}`);
    } else {
      setCustomText(`📰 ${item.title}\n\n${item.summary || ''}\n\n👉 Lire l'article : ${item.original_url}`);
    }
    // Sélectionne par défaut tous les comptes actifs
    setSelectedAccountIds(accounts.filter((a) => a.is_active).map((a) => a.id));
  };

  return (
    <div className="space-y-6 animate-fadeIn">
      {/* En-tête */}
      <div className="flex flex-col md:flex-row md:items-center justify-between gap-4">
        <div>
          <h1 className="text-3xl font-bold tracking-tight text-white flex items-center gap-3">
            <span>Flux de Curation</span>
            <span className="text-xs px-2.5 py-1 rounded-full bg-amber-500/10 text-amber-400 border border-amber-500/20 font-medium">
              {items.length} éléments
            </span>
          </h1>
          <p className="text-slate-400 text-sm mt-1">
            Revue éditoriale des articles et offres d'emploi extraits par le scraper
          </p>
        </div>

        {/* Filtres de Statut */}
        <div className="flex items-center gap-2 bg-slate-900 p-1.5 rounded-xl border border-slate-800 self-start md:self-auto">
          {(['NEW', 'QUEUED', 'REJECTED'] as CurationStatus[]).map((st) => (
            <button
              key={st}
              onClick={() => setStatusFilter(st)}
              className={`px-3 py-1.5 rounded-lg text-xs font-semibold transition ${
                statusFilter === st
                  ? 'bg-indigo-600 text-white shadow'
                  : 'text-slate-400 hover:text-slate-200'
              }`}
            >
              {st === 'NEW' && 'Nouveaux'}
              {st === 'QUEUED' && 'En file'}
              {st === 'REJECTED' && 'Rejetés'}
            </button>
          ))}
        </div>
      </div>

      {/* Barre de filtres de Catégories */}
      <div className="flex items-center gap-3 pb-2 border-b border-slate-800 text-sm">
        <Filter className="w-4 h-4 text-slate-400" />
        <span className="text-slate-400 text-xs font-medium uppercase">Filtrer par type :</span>
        <button
          onClick={() => setCategoryFilter('ALL')}
          className={`px-3 py-1 rounded-full text-xs font-medium transition ${
            categoryFilter === 'ALL'
              ? 'bg-slate-700 text-white'
              : 'text-slate-400 hover:bg-slate-800'
          }`}
        >
          Tous
        </button>
        <button
          onClick={() => setCategoryFilter('NEWS')}
          className={`flex items-center gap-1.5 px-3 py-1 rounded-full text-xs font-medium transition ${
            categoryFilter === 'NEWS'
              ? 'bg-blue-600/30 text-blue-300 border border-blue-500/40'
              : 'text-slate-400 hover:bg-slate-800'
          }`}
        >
          <Newspaper className="w-3.5 h-3.5 text-blue-400" />
          <span>Actualités</span>
        </button>
        <button
          onClick={() => setCategoryFilter('JOB_OFFER')}
          className={`flex items-center gap-1.5 px-3 py-1 rounded-full text-xs font-medium transition ${
            categoryFilter === 'JOB_OFFER'
              ? 'bg-purple-600/30 text-purple-300 border border-purple-500/40'
              : 'text-slate-400 hover:bg-slate-800'
          }`}
        >
          <Briefcase className="w-3.5 h-3.5 text-purple-400" />
          <span>Offres d'Emploi</span>
        </button>
      </div>

      {/* Liste des éléments */}
      {isLoading ? (
        <div className="flex h-64 items-center justify-center">
          <div className="h-8 w-8 animate-spin rounded-full border-4 border-indigo-500 border-t-transparent" />
        </div>
      ) : items.length === 0 ? (
        <div className="bg-slate-900/60 border border-slate-800 rounded-2xl p-12 text-center">
          <Layers className="w-12 h-12 text-slate-600 mx-auto mb-3" />
          <h3 className="text-slate-300 font-semibold">Aucun élément dans cette vue</h3>
          <p className="text-slate-500 text-sm mt-1">
            Lancez un scraping manuel depuis l'onglet Sources pour alimenter le flux.
          </p>
        </div>
      ) : (
        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-5">
          {items.map((item) => {
            const isJob = item.metadata?.item_type === 'JobOffer';
            return (
              <div
                key={item._id || item.url_hash}
                className="bg-slate-900 border border-slate-800 rounded-2xl overflow-hidden shadow-lg flex flex-col justify-between hover:border-slate-700 transition"
              >
                <div>
                  {/* Image éventuelle */}
                  {item.image_url && (
                    <div className="h-44 w-full bg-slate-950 overflow-hidden relative">
                      <img
                        src={item.image_url}
                        alt={item.title}
                        className="w-full h-full object-cover opacity-90 hover:opacity-100 transition duration-300"
                        onError={(e) => {
                          (e.target as HTMLElement).style.display = 'none';
                        }}
                      />
                    </div>
                  )}

                  <div className="p-5 space-y-3">
                    {/* Badge catégorie et date */}
                    <div className="flex items-center justify-between text-xs">
                      <span
                        className={`flex items-center gap-1.5 px-2.5 py-0.5 rounded-full font-semibold ${
                          isJob
                            ? 'bg-purple-500/20 text-purple-300 border border-purple-500/30'
                            : 'bg-blue-500/20 text-blue-300 border border-blue-500/30'
                        }`}
                      >
                        {isJob ? <Briefcase className="w-3 h-3" /> : <Newspaper className="w-3 h-3" />}
                        {isJob ? 'Emploi' : 'Actualité'}
                      </span>
                      <span className="text-slate-500">
                        {new Date(item.scraped_at).toLocaleDateString('fr-FR', {
                          day: 'numeric',
                          month: 'short',
                          hour: '2-digit',
                          minute: '2-digit',
                        })}
                      </span>
                    </div>

                    {/* Titre avec lien */}
                    <h3 className="font-bold text-white text-base leading-snug line-clamp-2 hover:text-indigo-400 transition">
                      <a href={item.original_url} target="_blank" rel="noreferrer" className="flex items-start gap-1">
                        <span>{item.title}</span>
                        <ExternalLink className="w-3.5 h-3.5 flex-shrink-0 mt-1 text-slate-400" />
                      </a>
                    </h3>

                    {/* Métadonnées spécifiques */}
                    {isJob && item.metadata?.item_type === 'JobOffer' && (
                      <div className="flex flex-wrap gap-2 text-xs text-slate-300 pt-1">
                        {item.metadata.company && (
                          <span className="flex items-center gap-1 bg-slate-800 px-2 py-0.5 rounded">
                            <Building className="w-3 h-3 text-slate-400" />
                            {item.metadata.company}
                          </span>
                        )}
                        {item.metadata.location && (
                          <span className="flex items-center gap-1 bg-slate-800 px-2 py-0.5 rounded">
                            <MapPin className="w-3 h-3 text-slate-400" />
                            {item.metadata.location}
                          </span>
                        )}
                        {item.metadata.contract_type && (
                          <span className="bg-indigo-950/60 text-indigo-300 px-2 py-0.5 rounded font-medium border border-indigo-500/20">
                            {item.metadata.contract_type}
                          </span>
                        )}
                      </div>
                    )}

                    {!isJob && item.metadata?.item_type === 'News' && item.metadata.author && (
                      <div className="flex items-center gap-1 text-xs text-slate-400">
                        <User className="w-3 h-3 text-slate-500" />
                        <span>Par {item.metadata.author}</span>
                      </div>
                    )}

                    {/* Résumé */}
                    {item.summary && (
                      <p className="text-slate-400 text-xs line-clamp-3 leading-relaxed">
                        {item.summary}
                      </p>
                    )}
                  </div>
                </div>

                {/* Actions Curation */}
                <div className="p-4 bg-slate-950/50 border-t border-slate-800 flex items-center justify-between gap-3">
                  {statusFilter === 'NEW' && (
                    <>
                      <button
                        onClick={() =>
                          item._id &&
                          updateStatusMutation.mutate({ id: item._id, status: 'REJECTED' })
                        }
                        className="flex-1 flex items-center justify-center gap-1.5 py-2 px-3 bg-slate-800 hover:bg-red-950/40 hover:text-red-300 hover:border-red-500/30 text-slate-300 text-xs font-medium rounded-xl border border-slate-700 transition"
                      >
                        <XCircle className="w-4 h-4 text-red-400" />
                        <span>Rejeter</span>
                      </button>
                      <button
                        onClick={() => handleOpenQueueModal(item)}
                        className="flex-1 flex items-center justify-center gap-1.5 py-2 px-3 bg-indigo-600 hover:bg-indigo-500 text-white text-xs font-semibold rounded-xl transition shadow-lg shadow-indigo-600/20"
                      >
                        <PlusCircle className="w-4 h-4" />
                        <span>En file</span>
                      </button>
                    </>
                  )}

                  {statusFilter === 'REJECTED' && (
                    <button
                      onClick={() =>
                        item._id &&
                        updateStatusMutation.mutate({ id: item._id, status: 'NEW' })
                      }
                      className="w-full flex items-center justify-center gap-1.5 py-2 px-3 bg-slate-800 hover:bg-slate-700 text-slate-300 text-xs font-medium rounded-xl border border-slate-700 transition"
                    >
                      <Check className="w-4 h-4 text-emerald-400" />
                      <span>Réintégrer dans le flux</span>
                    </button>
                  )}

                  {statusFilter === 'QUEUED' && (
                    <div className="w-full text-center text-xs text-indigo-400 font-medium py-1">
                      ✓ Déjà planifié en file d'attente
                    </div>
                  )}
                </div>
              </div>
            );
          })}
        </div>
      )}

      {/* Modal de Planification en File d'Attente */}
      {selectedItem && (
        <div className="fixed inset-0 bg-black/70 backdrop-blur-sm z-50 flex items-center justify-center p-4">
          <div className="bg-slate-900 border border-slate-800 rounded-2xl max-w-lg w-full p-6 shadow-2xl space-y-5 animate-scaleUp">
            <div className="flex items-center justify-between pb-3 border-b border-slate-800">
              <h3 className="text-lg font-bold text-white flex items-center gap-2">
                <PlusCircle className="w-5 h-5 text-indigo-400" />
                <span>Mettre en File d'Attente</span>
              </h3>
              <button
                onClick={() => setSelectedItem(null)}
                className="text-slate-400 hover:text-white"
              >
                ✕
              </button>
            </div>

            <div className="space-y-4 text-sm">
              <div>
                <label className="block text-xs font-semibold text-slate-400 uppercase mb-1">
                  Titre du contenu
                </label>
                <p className="text-slate-200 font-medium text-xs bg-slate-950 p-2.5 rounded-lg border border-slate-800">
                  {selectedItem.title}
                </p>
              </div>

              <div>
                <label className="block text-xs font-semibold text-slate-400 uppercase mb-1">
                  Priorité de diffusion (-10 à +10)
                </label>
                <div className="flex items-center gap-3">
                  <input
                    type="range"
                    min="-10"
                    max="10"
                    value={priority}
                    onChange={(e) => setPriority(parseInt(e.target.value, 10))}
                    className="w-full accent-indigo-500 cursor-pointer"
                  />
                  <span className="font-mono font-bold text-indigo-400 w-8 text-center">
                    {priority > 0 ? `+${priority}` : priority}
                  </span>
                </div>
                <p className="text-[11px] text-slate-500 mt-1">
                  Les publications avec la plus haute priorité sont traitées en premier.
                </p>
              </div>

              <div>
                <label className="block text-xs font-semibold text-slate-400 uppercase mb-1">
                  Texte à diffuser
                </label>
                <textarea
                  rows={4}
                  value={customText}
                  onChange={(e) => setCustomText(e.target.value)}
                  className="w-full bg-slate-950 border border-slate-800 rounded-xl p-3 text-xs text-white focus:outline-none focus:border-indigo-500 transition"
                  placeholder="Texte personnalisé pour les réseaux sociaux..."
                />
              </div>

              <div>
                <label className="block text-xs font-semibold text-slate-400 uppercase mb-1">
                  Comptes Sociaux Cibles
                </label>
                <div className="space-y-2 max-h-36 overflow-y-auto">
                  {accounts.map((acc) => (
                    <label
                      key={acc.id}
                      className="flex items-center justify-between p-2 rounded-lg bg-slate-950 border border-slate-800/80 hover:border-slate-700 cursor-pointer text-xs"
                    >
                      <span className="text-slate-200 flex items-center gap-2">
                        <span className="font-semibold text-indigo-300">[{acc.platform}]</span>
                        <span>{acc.label}</span>
                      </span>
                      <input
                        type="checkbox"
                        checked={selectedAccountIds.includes(acc.id)}
                        onChange={(e) => {
                          if (e.target.checked) {
                            setSelectedAccountIds([...selectedAccountIds, acc.id]);
                          } else {
                            setSelectedAccountIds(selectedAccountIds.filter((id) => id !== acc.id));
                          }
                        }}
                        className="rounded accent-indigo-500"
                      />
                    </label>
                  ))}
                </div>
              </div>
            </div>

            <div className="flex items-center justify-end gap-3 pt-3 border-t border-slate-800">
              <button
                onClick={() => setSelectedItem(null)}
                className="px-4 py-2 text-xs font-medium text-slate-400 hover:text-white"
              >
                Annuler
              </button>
              <button
                disabled={queueMutation.isPending}
                onClick={() =>
                  selectedItem._id &&
                  queueMutation.mutate({
                    id: selectedItem._id,
                    priority,
                    customText,
                    targetAccountIds: selectedAccountIds,
                  })
                }
                className="px-5 py-2.5 bg-indigo-600 hover:bg-indigo-500 text-white text-xs font-semibold rounded-xl shadow-lg shadow-indigo-600/30 transition disabled:opacity-50"
              >
                {queueMutation.isPending ? 'En cours...' : 'Confirmer et Ajouter'}
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
};
