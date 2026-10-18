import React, { useState } from 'react';
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { apiClient } from '../api/client';
import { Publication, PublicationStatus } from '../types';
import {
  Send,
  RotateCw,
  Trash2,
  Edit3,
  Clock,
  CheckCircle,
  AlertCircle,
  AlertTriangle,
  Play,
  Layers,
  Sparkles,
} from 'lucide-react';

export const QueueView: React.FC = () => {
  const queryClient = useQueryClient();
  const [statusFilter, setStatusFilter] = useState<string>('ALL');

  // État de l'édition d'une publication
  const [editingPub, setEditingPub] = useState<Publication | null>(null);
  const [editText, setEditText] = useState<string>('');
  const [editPriority, setEditPriority] = useState<number>(0);
  const [deliveryTexts, setDeliveryTexts] = useState<{ [accId: string]: string }>({});

  const { data: publications = [], isLoading } = useQuery({
    queryKey: ['publications', statusFilter],
    queryFn: () =>
      apiClient.getPublications({
        status: statusFilter !== 'ALL' ? (statusFilter as PublicationStatus) : undefined,
      }),
  });

  // Action : Publier maintenant (immédiat)
  const publishNowMutation = useMutation({
    mutationFn: (id: string) => apiClient.publishNow(id),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['publications'] });
      queryClient.invalidateQueries({ queryKey: ['dashboard-stats'] });
    },
  });

  // Action : Réessayer les échecs
  const retryMutation = useMutation({
    mutationFn: (id: string) => apiClient.retryPublication(id),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['publications'] });
      queryClient.invalidateQueries({ queryKey: ['dashboard-stats'] });
    },
  });

  // Action : Supprimer
  const deleteMutation = useMutation({
    mutationFn: (id: string) => apiClient.deletePublication(id),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['publications'] });
      queryClient.invalidateQueries({ queryKey: ['dashboard-stats'] });
    },
  });

  // Action : Mise à jour
  const updateMutation = useMutation({
    mutationFn: ({
      id,
      priority,
      defaultText,
      deliveries,
    }: {
      id: string;
      priority: number;
      defaultText: string;
      deliveries: { account_id: string; custom_text?: string }[];
    }) =>
      apiClient.updatePublication(id, {
        priority,
        default_text: defaultText,
        deliveries,
      }),
    onSuccess: () => {
      setEditingPub(null);
      queryClient.invalidateQueries({ queryKey: ['publications'] });
    },
  });

  const handleOpenEdit = (pub: Publication) => {
    setEditingPub(pub);
    setEditText(pub.default_text);
    setEditPriority(pub.priority);
    const dMap: { [accId: string]: string } = {};
    pub.deliveries.forEach((d) => {
      dMap[d.account_id] = d.custom_text || '';
    });
    setDeliveryTexts(dMap);
  };

  const getStatusBadge = (status: PublicationStatus) => {
    switch (status) {
      case 'PENDING':
        return (
          <span className="flex items-center gap-1.5 px-2.5 py-1 rounded-full text-xs font-semibold bg-amber-500/10 text-amber-400 border border-amber-500/20">
            <Clock className="w-3.5 h-3.5" /> En attente
          </span>
        );
      case 'PROCESSING':
        return (
          <span className="flex items-center gap-1.5 px-2.5 py-1 rounded-full text-xs font-semibold bg-blue-500/10 text-blue-400 border border-blue-500/20 animate-pulse">
            <RotateCw className="w-3.5 h-3.5 animate-spin" /> En cours
          </span>
        );
      case 'PUBLISHED':
        return (
          <span className="flex items-center gap-1.5 px-2.5 py-1 rounded-full text-xs font-semibold bg-emerald-500/10 text-emerald-400 border border-emerald-500/20">
            <CheckCircle className="w-3.5 h-3.5" /> Publié
          </span>
        );
      case 'PARTIAL_FAILURE':
        return (
          <span className="flex items-center gap-1.5 px-2.5 py-1 rounded-full text-xs font-semibold bg-orange-500/10 text-orange-400 border border-orange-500/20">
            <AlertTriangle className="w-3.5 h-3.5" /> Succès partiel
          </span>
        );
      case 'FAILED':
        return (
          <span className="flex items-center gap-1.5 px-2.5 py-1 rounded-full text-xs font-semibold bg-red-500/10 text-red-400 border border-red-500/20">
            <AlertCircle className="w-3.5 h-3.5" /> Échec
          </span>
        );
    }
  };

  return (
    <div className="space-y-6 animate-fadeIn">
      {/* En-tête */}
      <div className="flex flex-col md:flex-row md:items-center justify-between gap-4">
        <div>
          <h1 className="text-3xl font-bold tracking-tight text-white flex items-center gap-3">
            <span>File d'Attente des Publications</span>
            <span className="text-xs px-2.5 py-1 rounded-full bg-indigo-500/10 text-indigo-400 border border-indigo-500/20 font-medium">
              {publications.length} éléments
            </span>
          </h1>
          <p className="text-slate-400 text-sm mt-1">
            Gestion de la file d'attente prioritaire et suivi des diffusions multi-plateformes
          </p>
        </div>

        {/* Filtres de statut */}
        <div className="flex flex-wrap items-center gap-1.5 bg-slate-900 p-1.5 rounded-xl border border-slate-800">
          {['ALL', 'PENDING', 'PUBLISHED', 'PARTIAL_FAILURE', 'FAILED'].map((st) => (
            <button
              key={st}
              onClick={() => setStatusFilter(st)}
              className={`px-3 py-1.5 rounded-lg text-xs font-semibold transition ${
                statusFilter === st
                  ? 'bg-indigo-600 text-white shadow'
                  : 'text-slate-400 hover:text-slate-200'
              }`}
            >
              {st === 'ALL' && 'Toutes'}
              {st === 'PENDING' && 'En attente'}
              {st === 'PUBLISHED' && 'Publiées'}
              {st === 'PARTIAL_FAILURE' && 'Partielles'}
              {st === 'FAILED' && 'Échecs'}
            </button>
          ))}
        </div>
      </div>

      {/* Liste des publications */}
      {isLoading ? (
        <div className="flex h-64 items-center justify-center">
          <div className="h-8 w-8 animate-spin rounded-full border-4 border-indigo-500 border-t-transparent" />
        </div>
      ) : publications.length === 0 ? (
        <div className="bg-slate-900/60 border border-slate-800 rounded-2xl p-12 text-center">
          <Layers className="w-12 h-12 text-slate-600 mx-auto mb-3" />
          <h3 className="text-slate-300 font-semibold">Aucune publication dans cette liste</h3>
          <p className="text-slate-500 text-sm mt-1">
            Mettez des articles en file d'attente depuis l'onglet Curation.
          </p>
        </div>
      ) : (
        <div className="space-y-4">
          {publications.map((pub) => (
            <div
              key={pub._id}
              className="bg-slate-900 border border-slate-800 rounded-2xl p-5 shadow-lg space-y-4 hover:border-slate-700 transition"
            >
              {/* Entête de carte : Statut, priorité, date */}
              <div className="flex flex-wrap items-center justify-between gap-3 pb-3 border-b border-slate-800/80">
                <div className="flex items-center gap-3">
                  {getStatusBadge(pub.status)}
                  <span className="text-xs px-2 py-0.5 rounded bg-slate-800 text-indigo-300 font-mono font-bold">
                    Priorité : {pub.priority > 0 ? `+${pub.priority}` : pub.priority}
                  </span>
                </div>
                <div className="text-xs text-slate-400 flex items-center gap-2">
                  <span>Créé le {new Date(pub.created_at).toLocaleString('fr-FR')}</span>
                  {pub.published_at && (
                    <span className="text-emerald-400">
                      • Publié le {new Date(pub.published_at).toLocaleTimeString('fr-FR')}
                    </span>
                  )}
                </div>
              </div>

              {/* Contenu textuel & médias */}
              <div className="grid grid-cols-1 lg:grid-cols-3 gap-4">
                <div className="lg:col-span-2 space-y-2">
                  <span className="text-[11px] font-semibold text-slate-400 uppercase">
                    Texte par défaut
                  </span>
                  <p className="text-sm text-slate-200 whitespace-pre-line bg-slate-950 p-3.5 rounded-xl border border-slate-800/60 font-sans">
                    {pub.default_text}
                  </p>
                </div>

                {/* Livraisons par réseau social (Deliveries) */}
                <div className="space-y-2">
                  <span className="text-[11px] font-semibold text-slate-400 uppercase">
                    Canaux de diffusion ({pub.deliveries.length})
                  </span>
                  <div className="space-y-2">
                    {pub.deliveries.map((del, dIdx) => (
                      <div
                        key={dIdx}
                        className="bg-slate-950 p-2.5 rounded-xl border border-slate-800/80 text-xs space-y-1"
                      >
                        <div className="flex items-center justify-between">
                          <span className="font-bold text-indigo-400">[{del.platform}]</span>
                          <span
                            className={`px-2 py-0.5 rounded text-[10px] font-semibold ${
                              del.status === 'SENT'
                                ? 'bg-emerald-500/20 text-emerald-300'
                                : del.status === 'FAILED'
                                ? 'bg-red-500/20 text-red-300'
                                : 'bg-amber-500/20 text-amber-300'
                            }`}
                          >
                            {del.status}
                          </span>
                        </div>
                        {del.external_post_id && (
                          <div className="text-[11px] text-slate-400 truncate">
                            ID externe : <code className="text-slate-300">{del.external_post_id}</code>
                          </div>
                        )}
                        {del.error_message && (
                          <div className="text-[11px] text-red-400 bg-red-950/30 p-1 rounded border border-red-900/40">
                            {del.error_message}
                          </div>
                        )}
                      </div>
                    ))}
                  </div>
                </div>
              </div>

              {/* Actions de barre d'outils */}
              <div className="pt-2 flex flex-wrap items-center justify-between gap-3 border-t border-slate-800/60">
                <div className="flex items-center gap-2">
                  {/* Bouton Publier Maintenant */}
                  {pub.status !== 'PUBLISHED' && (
                    <button
                      onClick={() => pub._id && publishNowMutation.mutate(pub._id)}
                      disabled={publishNowMutation.isPending}
                      className="flex items-center gap-1.5 px-3 py-1.5 bg-indigo-600 hover:bg-indigo-500 text-white text-xs font-semibold rounded-lg shadow-sm transition"
                    >
                      <Play className="w-3.5 h-3.5 fill-current" />
                      <span>Publier maintenant</span>
                    </button>
                  )}

                  {/* Bouton Réessayer en cas d'échec */}
                  {(pub.status === 'FAILED' || pub.status === 'PARTIAL_FAILURE') && (
                    <button
                      onClick={() => pub._id && retryMutation.mutate(pub._id)}
                      disabled={retryMutation.isPending}
                      className="flex items-center gap-1.5 px-3 py-1.5 bg-amber-600/20 hover:bg-amber-600/30 text-amber-300 border border-amber-500/30 text-xs font-semibold rounded-lg transition"
                    >
                      <RotateCw className="w-3.5 h-3.5" />
                      <span>Relancer les échecs</span>
                    </button>
                  )}

                  {/* Bouton Éditer */}
                  <button
                    onClick={() => handleOpenEdit(pub)}
                    className="flex items-center gap-1.5 px-3 py-1.5 bg-slate-800 hover:bg-slate-700 text-slate-300 text-xs font-medium rounded-lg border border-slate-700 transition"
                  >
                    <Edit3 className="w-3.5 h-3.5" />
                    <span>Modifier</span>
                  </button>
                </div>

                {/* Bouton Supprimer */}
                <button
                  onClick={() => pub._id && deleteMutation.mutate(pub._id)}
                  className="flex items-center gap-1.5 px-3 py-1.5 bg-slate-900 hover:bg-red-950/40 text-slate-400 hover:text-red-300 hover:border-red-500/30 text-xs font-medium rounded-lg border border-slate-800 transition"
                >
                  <Trash2 className="w-3.5 h-3.5 text-red-400" />
                  <span>Supprimer</span>
                </button>
              </div>
            </div>
          ))}
        </div>
      )}

      {/* Modal d'édition */}
      {editingPub && (
        <div className="fixed inset-0 bg-black/70 backdrop-blur-sm z-50 flex items-center justify-center p-4">
          <div className="bg-slate-900 border border-slate-800 rounded-2xl max-w-xl w-full p-6 shadow-2xl space-y-5 animate-scaleUp">
            <div className="flex items-center justify-between pb-3 border-b border-slate-800">
              <h3 className="text-lg font-bold text-white flex items-center gap-2">
                <Edit3 className="w-5 h-5 text-indigo-400" />
                <span>Modifier la Publication</span>
              </h3>
              <button
                onClick={() => setEditingPub(null)}
                className="text-slate-400 hover:text-white"
              >
                ✕
              </button>
            </div>

            <div className="space-y-4 text-sm">
              <div>
                <label className="block text-xs font-semibold text-slate-400 uppercase mb-1">
                  Priorité (-10 à +10)
                </label>
                <input
                  type="number"
                  min="-10"
                  max="10"
                  value={editPriority}
                  onChange={(e) => setEditPriority(parseInt(e.target.value, 10))}
                  className="w-full bg-slate-950 border border-slate-800 rounded-xl p-2.5 text-xs text-white"
                />
              </div>

              <div>
                <label className="block text-xs font-semibold text-slate-400 uppercase mb-1">
                  Texte par défaut
                </label>
                <textarea
                  rows={4}
                  value={editText}
                  onChange={(e) => setEditText(e.target.value)}
                  className="w-full bg-slate-950 border border-slate-800 rounded-xl p-3 text-xs text-white focus:outline-none focus:border-indigo-500"
                />
              </div>

              {/* Textes personnalisés par livraison */}
              {editingPub.deliveries.length > 0 && (
                <div className="space-y-3">
                  <label className="block text-xs font-semibold text-slate-400 uppercase">
                    Personnalisation par Réseau Social
                  </label>
                  {editingPub.deliveries.map((del) => (
                    <div key={del.account_id} className="space-y-1">
                      <span className="text-xs font-semibold text-indigo-400">
                        Canal : {del.platform}
                      </span>
                      <textarea
                        rows={2}
                        value={deliveryTexts[del.account_id] || ''}
                        onChange={(e) =>
                          setDeliveryTexts({
                            ...deliveryTexts,
                            [del.account_id]: e.target.value,
                          })
                        }
                        placeholder={`Texte spécifique pour ${del.platform} (laisser vide pour utiliser le texte par défaut)...`}
                        className="w-full bg-slate-950 border border-slate-800 rounded-xl p-2.5 text-xs text-white focus:outline-none focus:border-indigo-500"
                      />
                    </div>
                  ))}
                </div>
              )}
            </div>

            <div className="flex items-center justify-end gap-3 pt-3 border-t border-slate-800">
              <button
                onClick={() => setEditingPub(null)}
                className="px-4 py-2 text-xs font-medium text-slate-400 hover:text-white"
              >
                Annuler
              </button>
              <button
                onClick={() => {
                  if (editingPub._id) {
                    const deliveriesPayload = Object.entries(deliveryTexts).map(
                      ([account_id, custom_text]) => ({
                        account_id,
                        custom_text: custom_text.trim() || undefined,
                      })
                    );
                    updateMutation.mutate({
                      id: editingPub._id,
                      priority: editPriority,
                      defaultText: editText,
                      deliveries: deliveriesPayload,
                    });
                  }
                }}
                className="px-5 py-2.5 bg-indigo-600 hover:bg-indigo-500 text-white text-xs font-semibold rounded-xl shadow-lg transition"
              >
                Enregistrer les modifications
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
};
