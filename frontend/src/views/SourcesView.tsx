import React, { useState } from 'react';
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { apiClient } from '../api/client';
import { Source, SourceCategory } from '../types';
import {
  Globe2,
  Plus,
  Play,
  Trash2,
  Edit3,
  AlertTriangle,
  CheckCircle,
  XCircle,
  Briefcase,
  Newspaper,
  ExternalLink,
  RotateCw,
  Sliders,
  Cpu,
} from 'lucide-react';

export const SourcesView: React.FC = () => {
  const queryClient = useQueryClient();
  const [isFormModalOpen, setIsFormModalOpen] = useState(false);
  const [editingSource, setEditingSource] = useState<Source | null>(null);
  const [deletingSource, setDeletingSource] = useState<Source | null>(null);
  const [scrapeFeedback, setScrapeFeedback] = useState<string | null>(null);

  // Formulaire source (création & édition)
  const [name, setName] = useState('');
  const [baseUrl, setBaseUrl] = useState('');
  const [category, setCategory] = useState<SourceCategory>('NEWS');
  const [cronSchedule, setCronSchedule] = useState('0 */30 * * * *');
  const [itemContainer, setItemContainer] = useState('article');
  const [titleSelector, setTitleSelector] = useState('h2, .title');
  const [urlSelector, setUrlSelector] = useState('a');
  const [summarySelector, setSummarySelector] = useState('p');
  const [imageSelector, setImageSelector] = useState('img');
  const [authorSelector, setAuthorSelector] = useState('.author');
  const [companySelector, setCompanySelector] = useState('.company');
  const [locationSelector, setLocationSelector] = useState('.location');

  // Configuration Headless Chromium / JS Rendu
  const [renderJs, setRenderJs] = useState(false);
  const [waitForSelector, setWaitForSelector] = useState('');
  const [waitDurationMs, setWaitDurationMs] = useState(2000);

  const { data: sources = [], isLoading } = useQuery({
    queryKey: ['sources'],
    queryFn: apiClient.getSources,
  });

  // Action : Scraper maintenant
  const scrapeNowMutation = useMutation({
    mutationFn: (id: string) => apiClient.scrapeSourceNow(id),
    onSuccess: (data) => {
      setScrapeFeedback(
        `Scraping réussi (${data.source_name}) : ${data.total_found} trouvés, ${data.new_inserted} nouveaux insérés, ${data.duplicates_skipped} doublons ignorés.`
      );
      queryClient.invalidateQueries({ queryKey: ['sources'] });
      queryClient.invalidateQueries({ queryKey: ['scraped-items'] });
      queryClient.invalidateQueries({ queryKey: ['dashboard-stats'] });
      setTimeout(() => setScrapeFeedback(null), 7000);
    },
    onError: (err: any) => {
      setScrapeFeedback(`Erreur lors du scraping : ${err.response?.data?.error || err.message}`);
    },
  });

  // Action : Basculer actif/inactif
  const toggleActiveMutation = useMutation({
    mutationFn: ({ id, is_active }: { id: string; is_active: boolean }) =>
      apiClient.updateSource(id, { is_active }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['sources'] });
      queryClient.invalidateQueries({ queryKey: ['dashboard-stats'] });
    },
  });

  // Action : Supprimer
  const deleteMutation = useMutation({
    mutationFn: (id: string) => apiClient.deleteSource(id),
    onSuccess: () => {
      setDeletingSource(null);
      queryClient.invalidateQueries({ queryKey: ['sources'] });
      queryClient.invalidateQueries({ queryKey: ['dashboard-stats'] });
    },
  });

  // Action : Créer une source
  const createMutation = useMutation({
    mutationFn: (newSource: Partial<Source>) => apiClient.createSource(newSource),
    onSuccess: () => {
      setIsFormModalOpen(false);
      resetForm();
      queryClient.invalidateQueries({ queryKey: ['sources'] });
      queryClient.invalidateQueries({ queryKey: ['dashboard-stats'] });
    },
  });

  // Action : Modifier une source
  const updateMutation = useMutation({
    mutationFn: ({ id, data }: { id: string; data: Partial<Source> }) =>
      apiClient.updateSource(id, data),
    onSuccess: () => {
      setIsFormModalOpen(false);
      setEditingSource(null);
      resetForm();
      queryClient.invalidateQueries({ queryKey: ['sources'] });
      queryClient.invalidateQueries({ queryKey: ['dashboard-stats'] });
    },
  });

  const resetForm = () => {
    setName('');
    setBaseUrl('');
    setCategory('NEWS');
    setCronSchedule('0 */30 * * * *');
    setItemContainer('article');
    setTitleSelector('h2, .title');
    setUrlSelector('a');
    setSummarySelector('p');
    setImageSelector('img');
    setAuthorSelector('.author');
    setCompanySelector('.company');
    setLocationSelector('.location');
    setRenderJs(false);
    setWaitForSelector('');
    setWaitDurationMs(2000);
  };

  const handleOpenCreate = () => {
    resetForm();
    setEditingSource(null);
    setIsFormModalOpen(true);
  };

  const handleOpenEdit = (src: Source) => {
    setEditingSource(src);
    setName(src.name);
    setBaseUrl(src.base_url);
    setCategory(src.category);
    setCronSchedule(src.cron_schedule);
    setItemContainer(src.scraper_config.item_container);
    setTitleSelector(src.scraper_config.title);
    setUrlSelector(src.scraper_config.url);
    setSummarySelector(src.scraper_config.summary || '');
    setImageSelector(src.scraper_config.image || '');
    setAuthorSelector(src.scraper_config.specific_fields?.author || '');
    setCompanySelector(src.scraper_config.specific_fields?.company || '');
    setLocationSelector(src.scraper_config.specific_fields?.location || '');
    setRenderJs(src.scraper_config.render_js || false);
    setWaitForSelector(src.scraper_config.wait_for_selector || '');
    setWaitDurationMs(src.scraper_config.wait_duration_ms || 2000);
    setIsFormModalOpen(true);
  };

  const handleFormSubmit = (e: React.FormEvent) => {
    e.preventDefault();

    const specific_fields: Record<string, string> = {};
    if (category === 'NEWS' && authorSelector) {
      specific_fields.author = authorSelector;
    }
    if (category === 'JOB_OFFER') {
      if (companySelector) specific_fields.company = companySelector;
      if (locationSelector) specific_fields.location = locationSelector;
    }

    const payload: Partial<Source> = {
      name,
      base_url: baseUrl,
      category,
      cron_schedule: cronSchedule,
      scraper_config: {
        item_container: itemContainer,
        title: titleSelector,
        url: urlSelector,
        summary: summarySelector || undefined,
        image: imageSelector || undefined,
        specific_fields: Object.keys(specific_fields).length ? specific_fields : undefined,
        render_js: renderJs,
        wait_for_selector: waitForSelector.trim() || undefined,
        wait_duration_ms: waitDurationMs ? Number(waitDurationMs) : undefined,
      },
    };

    if (editingSource && editingSource._id) {
      updateMutation.mutate({ id: editingSource._id, data: payload });
    } else {
      createMutation.mutate({
        ...payload,
        is_active: true,
      });
    }
  };

  return (
    <div className="space-y-6 animate-fadeIn">
      {/* En-tête */}
      <div className="flex flex-col md:flex-row md:items-center justify-between gap-4">
        <div>
          <h1 className="text-3xl font-bold tracking-tight text-white flex items-center gap-3">
            <span>Sources de Veille & Scraping</span>
            <span className="text-xs px-2.5 py-1 rounded-full bg-blue-500/10 text-blue-400 border border-blue-500/20 font-medium">
              {sources.length} sources
            </span>
          </h1>
          <p className="text-slate-400 text-sm mt-1">
            Cibles de collecte automatisée, rendu JavaScript Headless via Chromium (CDP), et planification
          </p>
        </div>

        <button
          onClick={handleOpenCreate}
          className="flex items-center gap-2 px-4 py-2.5 bg-indigo-600 hover:bg-indigo-500 text-white text-xs font-semibold rounded-xl transition shadow-lg shadow-indigo-600/20 self-start md:self-auto cursor-pointer"
        >
          <Plus className="w-4 h-4" />
          <span>Ajouter une Source</span>
        </button>
      </div>

      {/* Message feedback scraping */}
      {scrapeFeedback && (
        <div className="bg-slate-900 border border-indigo-500/40 text-indigo-300 p-4 rounded-xl text-xs flex items-center justify-between shadow-lg animate-fadeIn">
          <span>{scrapeFeedback}</span>
          <button onClick={() => setScrapeFeedback(null)} className="text-slate-400 hover:text-white cursor-pointer">
            ✕
          </button>
        </div>
      )}

      {/* Liste des Sources */}
      {isLoading ? (
        <div className="flex h-64 items-center justify-center">
          <div className="h-8 w-8 animate-spin rounded-full border-4 border-indigo-500 border-t-transparent" />
        </div>
      ) : sources.length === 0 ? (
        <div className="bg-slate-900/60 border border-slate-800 rounded-2xl p-12 text-center">
          <Globe2 className="w-12 h-12 text-slate-600 mx-auto mb-3" />
          <h3 className="text-slate-300 font-semibold">Aucune source enregistrée</h3>
          <p className="text-slate-500 text-sm mt-1 mb-4">
            Ajoutez un premier site d'actualité ou de recrutement pour débuter la veille.
          </p>
          <button
            onClick={handleOpenCreate}
            className="px-4 py-2 bg-indigo-600 hover:bg-indigo-500 text-white text-xs font-semibold rounded-xl cursor-pointer"
          >
            Configurer une source
          </button>
        </div>
      ) : (
        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-5">
          {sources.map((src) => {
            const isJob = src.category === 'JOB_OFFER';
            const isHeadless = !!src.scraper_config.render_js;

            return (
              <div
                key={src._id}
                className={`bg-slate-900 border rounded-2xl p-5 shadow-lg space-y-4 transition ${
                  src.is_active ? 'border-slate-800 hover:border-slate-700' : 'border-slate-800/40 opacity-70'
                }`}
              >
                <div className="flex items-center justify-between">
                  <div className="flex items-center gap-2">
                    <span
                      className={`flex items-center gap-1.5 px-2.5 py-0.5 rounded-full text-xs font-semibold ${
                        isJob
                          ? 'bg-purple-500/20 text-purple-300 border border-purple-500/30'
                          : 'bg-blue-500/20 text-blue-300 border border-blue-500/30'
                      }`}
                    >
                      {isJob ? <Briefcase className="w-3 h-3" /> : <Newspaper className="w-3 h-3" />}
                      {isJob ? "Offres d'Emploi" : 'Actualités'}
                    </span>

                    {/* Badge Mode Headless Chromium */}
                    {isHeadless ? (
                      <span className="flex items-center gap-1 px-2 py-0.5 rounded-full text-[10px] font-semibold bg-cyan-500/10 text-cyan-300 border border-cyan-500/20" title="Rendu JavaScript actif via Chromium Headless">
                        <Cpu className="w-3 h-3" />
                        <span>Chromium</span>
                      </span>
                    ) : (
                      <span className="flex items-center gap-1 px-2 py-0.5 rounded-full text-[10px] font-normal bg-slate-800/80 text-slate-400" title="Extraction HTTP rapide avec fallback automatique">
                        <span>HTTP/HTML</span>
                      </span>
                    )}
                  </div>

                  {/* Toggle Actif/Inactif */}
                  <button
                    onClick={() =>
                      src._id &&
                      toggleActiveMutation.mutate({ id: src._id, is_active: !src.is_active })
                    }
                    className={`flex items-center gap-1 px-2.5 py-1 rounded-lg text-xs font-medium transition cursor-pointer ${
                      src.is_active
                        ? 'bg-emerald-500/10 text-emerald-400 border border-emerald-500/20'
                        : 'bg-slate-800 text-slate-400'
                    }`}
                  >
                    {src.is_active ? <CheckCircle className="w-3 h-3" /> : <XCircle className="w-3 h-3" />}
                    <span>{src.is_active ? 'Actif' : 'Inactif'}</span>
                  </button>
                </div>

                <div>
                  <h3 className="text-base font-bold text-white">{src.name}</h3>
                  <a
                    href={src.base_url}
                    target="_blank"
                    rel="noreferrer"
                    className="text-xs text-indigo-400 hover:underline flex items-center gap-1 mt-1 truncate"
                  >
                    <span>{src.base_url}</span>
                    <ExternalLink className="w-3 h-3 flex-shrink-0" />
                  </a>
                </div>

                {/* Configuration des sélecteurs */}
                <div className="bg-slate-950 p-3 rounded-xl border border-slate-800/80 text-[11px] font-mono space-y-1 text-slate-300">
                  <div className="flex justify-between">
                    <span className="text-slate-500">Container:</span>
                    <span className="text-indigo-300 truncate max-w-[150px]">{src.scraper_config.item_container}</span>
                  </div>
                  <div className="flex justify-between">
                    <span className="text-slate-500">Titre:</span>
                    <span className="text-indigo-300 truncate max-w-[150px]">{src.scraper_config.title}</span>
                  </div>
                  <div className="flex justify-between">
                    <span className="text-slate-500">Lien:</span>
                    <span className="text-indigo-300 truncate max-w-[150px]">{src.scraper_config.url}</span>
                  </div>
                </div>

                <div className="text-xs text-slate-400 flex items-center justify-between">
                  <span>
                    Dernier scan :{' '}
                    {src.last_scraped_at
                      ? new Date(src.last_scraped_at).toLocaleTimeString('fr-FR')
                      : 'Jamais'}
                  </span>
                  <span className="font-mono text-[10px] bg-slate-800 px-1.5 py-0.5 rounded text-slate-300">
                    {src.cron_schedule}
                  </span>
                </div>

                {/* Actions */}
                <div className="pt-2 flex items-center justify-between gap-2 border-t border-slate-800">
                  <button
                    onClick={() => src._id && scrapeNowMutation.mutate(src._id)}
                    disabled={scrapeNowMutation.isPending}
                    className="flex-1 flex items-center justify-center gap-1.5 py-2 px-3 bg-indigo-600 hover:bg-indigo-500 text-white text-xs font-semibold rounded-xl transition shadow cursor-pointer"
                  >
                    <Play className="w-3 h-3 fill-current" />
                    <span>Scraper</span>
                  </button>

                  <button
                    onClick={() => handleOpenEdit(src)}
                    className="p-2 text-slate-400 hover:text-indigo-400 hover:bg-indigo-950/40 rounded-xl transition cursor-pointer"
                    title="Modifier la source"
                  >
                    <Edit3 className="w-4 h-4" />
                  </button>

                  <button
                    onClick={() => setDeletingSource(src)}
                    className="p-2 text-slate-400 hover:text-red-400 hover:bg-red-950/40 rounded-xl transition cursor-pointer"
                    title="Supprimer la source"
                  >
                    <Trash2 className="w-4 h-4" />
                  </button>
                </div>
              </div>
            );
          })}
        </div>
      )}

      {/* Modal Ajout / Modification Source */}
      {isFormModalOpen && (
        <div className="fixed inset-0 bg-black/75 backdrop-blur-sm z-50 flex items-center justify-center p-4 animate-fadeIn">
          <div className="bg-slate-900 border border-slate-800 rounded-2xl max-w-xl w-full p-6 shadow-2xl space-y-5 max-h-[90vh] overflow-y-auto animate-scaleUp">
            <div className="flex items-center justify-between pb-3 border-b border-slate-800">
              <h3 className="text-lg font-bold text-white flex items-center gap-2">
                {editingSource ? (
                  <>
                    <Edit3 className="w-5 h-5 text-indigo-400" />
                    <span>Modifier la Source</span>
                  </>
                ) : (
                  <>
                    <Globe2 className="w-5 h-5 text-indigo-400" />
                    <span>Nouvelle Cible de Scraping</span>
                  </>
                )}
              </h3>
              <button
                onClick={() => {
                  setIsFormModalOpen(false);
                  setEditingSource(null);
                }}
                className="text-slate-400 hover:text-white cursor-pointer"
              >
                ✕
              </button>
            </div>

            <form onSubmit={handleFormSubmit} className="space-y-4 text-xs">
              <div className="grid grid-cols-2 gap-3">
                <div>
                  <label className="block font-semibold text-slate-400 uppercase mb-1">
                    Nom du site
                  </label>
                  <input
                    type="text"
                    required
                    value={name}
                    onChange={(e) => setName(e.target.value)}
                    placeholder="Ex: TechCrunch News"
                    className="w-full bg-slate-950 border border-slate-800 rounded-xl p-2.5 text-white focus:outline-none focus:border-indigo-500"
                  />
                </div>
                <div>
                  <label className="block font-semibold text-slate-400 uppercase mb-1">
                    Catégorie
                  </label>
                  <select
                    value={category}
                    onChange={(e) => setCategory(e.target.value as SourceCategory)}
                    className="w-full bg-slate-950 border border-slate-800 rounded-xl p-2.5 text-white focus:outline-none focus:border-indigo-500"
                  >
                    <option value="NEWS">Actualités (NEWS)</option>
                    <option value="JOB_OFFER">Offres d'Emploi (JOB_OFFER)</option>
                  </select>
                </div>
              </div>

              <div>
                <label className="block font-semibold text-slate-400 uppercase mb-1">
                  URL cible
                </label>
                <input
                  type="url"
                  required
                  value={baseUrl}
                  onChange={(e) => setBaseUrl(e.target.value)}
                  placeholder="https://techcrunch.com/category/startups"
                  className="w-full bg-slate-950 border border-slate-800 rounded-xl p-2.5 text-white focus:outline-none focus:border-indigo-500"
                />
              </div>

              {/* Bloc Rendu JavaScript Headless (Chromium) */}
              <div className="p-4 bg-slate-950 rounded-xl border border-slate-800 space-y-3">
                <div className="flex items-center justify-between">
                  <span className="font-bold text-cyan-400 uppercase flex items-center gap-1.5">
                    <Cpu className="w-3.5 h-3.5" /> Rendu JavaScript (Headless Chromium)
                  </span>
                  <label className="relative inline-flex items-center cursor-pointer">
                    <input
                      type="checkbox"
                      checked={renderJs}
                      onChange={(e) => setRenderJs(e.target.checked)}
                      className="sr-only peer"
                    />
                    <div className="w-9 h-5 bg-slate-800 peer-focus:outline-none rounded-full peer peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:border-slate-300 after:border after:rounded-full after:h-4 after:w-4 after:transition-all peer-checked:bg-cyan-600"></div>
                  </label>
                </div>
                <p className="text-[11px] text-slate-400 leading-relaxed">
                  Active le navigateur headless pour exécuter le JavaScript côté client (CSR / React / Vue) ou récupérer les éléments injectés tardivement.
                </p>

                {renderJs && (
                  <div className="grid grid-cols-2 gap-3 pt-2 border-t border-slate-800/80 animate-fadeIn">
                    <div>
                      <label className="block text-slate-400 mb-1">Attendre sélecteur (optionnel)</label>
                      <input
                        type="text"
                        value={waitForSelector}
                        onChange={(e) => setWaitForSelector(e.target.value)}
                        placeholder="Par défaut: container"
                        className="w-full bg-slate-900 border border-slate-800 rounded-lg p-2 text-white"
                      />
                    </div>
                    <div>
                      <label className="block text-slate-400 mb-1">Délai d'hydratation (ms)</label>
                      <input
                        type="number"
                        min="500"
                        max="15000"
                        step="500"
                        value={waitDurationMs}
                        onChange={(e) => setWaitDurationMs(Number(e.target.value))}
                        placeholder="2000"
                        className="w-full bg-slate-900 border border-slate-800 rounded-lg p-2 text-white font-mono"
                      />
                    </div>
                  </div>
                )}
              </div>

              {/* Sélecteurs CSS */}
              <div className="p-4 bg-slate-950 rounded-xl border border-slate-800 space-y-3">
                <span className="font-bold text-indigo-300 uppercase flex items-center gap-1.5">
                  <Sliders className="w-3.5 h-3.5" /> Sélecteurs CSS de Scraping
                </span>

                <div className="grid grid-cols-2 gap-3">
                  <div>
                    <label className="block text-slate-400 mb-1">Conteneur d'élément</label>
                    <input
                      type="text"
                      required
                      value={itemContainer}
                      onChange={(e) => setItemContainer(e.target.value)}
                      placeholder="article, .post, .job-item"
                      className="w-full bg-slate-900 border border-slate-800 rounded-lg p-2 text-white"
                    />
                  </div>
                  <div>
                    <label className="block text-slate-400 mb-1">Sélecteur Titre</label>
                    <input
                      type="text"
                      required
                      value={titleSelector}
                      onChange={(e) => setTitleSelector(e.target.value)}
                      placeholder="h2, a.title"
                      className="w-full bg-slate-900 border border-slate-800 rounded-lg p-2 text-white"
                    />
                  </div>
                  <div>
                    <label className="block text-slate-400 mb-1">Sélecteur Lien URL</label>
                    <input
                      type="text"
                      required
                      value={urlSelector}
                      onChange={(e) => setUrlSelector(e.target.value)}
                      placeholder="a"
                      className="w-full bg-slate-900 border border-slate-800 rounded-lg p-2 text-white"
                    />
                  </div>
                  <div>
                    <label className="block text-slate-400 mb-1">Sélecteur Résumé</label>
                    <input
                      type="text"
                      value={summarySelector}
                      onChange={(e) => setSummarySelector(e.target.value)}
                      placeholder="p.excerpt, .description"
                      className="w-full bg-slate-900 border border-slate-800 rounded-lg p-2 text-white"
                    />
                  </div>
                  <div>
                    <label className="block text-slate-400 mb-1">Sélecteur Image</label>
                    <input
                      type="text"
                      value={imageSelector}
                      onChange={(e) => setImageSelector(e.target.value)}
                      placeholder="img"
                      className="w-full bg-slate-900 border border-slate-800 rounded-lg p-2 text-white"
                    />
                  </div>

                  {category === 'JOB_OFFER' ? (
                    <div>
                      <label className="block text-slate-400 mb-1">Sélecteur Entreprise</label>
                      <input
                        type="text"
                        value={companySelector}
                        onChange={(e) => setCompanySelector(e.target.value)}
                        placeholder=".company-name"
                        className="w-full bg-slate-900 border border-slate-800 rounded-lg p-2 text-white"
                      />
                    </div>
                  ) : (
                    <div>
                      <label className="block text-slate-400 mb-1">Sélecteur Auteur</label>
                      <input
                        type="text"
                        value={authorSelector}
                        onChange={(e) => setAuthorSelector(e.target.value)}
                        placeholder=".author-name"
                        className="w-full bg-slate-900 border border-slate-800 rounded-lg p-2 text-white"
                      />
                    </div>
                  )}
                </div>
              </div>

              <div>
                <label className="block font-semibold text-slate-400 uppercase mb-1">
                  Expression Cron
                </label>
                <input
                  type="text"
                  required
                  value={cronSchedule}
                  onChange={(e) => setCronSchedule(e.target.value)}
                  placeholder="0 */30 * * * *"
                  className="w-full bg-slate-950 border border-slate-800 rounded-xl p-2.5 text-white font-mono"
                />
              </div>

              <div className="flex items-center justify-end gap-3 pt-3 border-t border-slate-800">
                <button
                  type="button"
                  onClick={() => {
                    setIsFormModalOpen(false);
                    setEditingSource(null);
                  }}
                  className="px-4 py-2 text-slate-400 hover:text-white cursor-pointer"
                >
                  Annuler
                </button>
                <button
                  type="submit"
                  disabled={createMutation.isPending || updateMutation.isPending}
                  className="flex items-center gap-2 px-5 py-2.5 bg-indigo-600 hover:bg-indigo-500 disabled:opacity-50 text-white font-semibold rounded-xl shadow-lg transition cursor-pointer"
                >
                  {(createMutation.isPending || updateMutation.isPending) && (
                    <RotateCw className="w-3.5 h-3.5 animate-spin" />
                  )}
                  <span>
                    {editingSource
                      ? updateMutation.isPending
                        ? 'Enregistrement...'
                        : 'Mettre à jour la source'
                      : createMutation.isPending
                      ? 'Création...'
                      : 'Créer la source'}
                  </span>
                </button>
              </div>
            </form>
          </div>
        </div>
      )}

      {/* Dialog Moderne de Confirmation de Suppression */}
      {deletingSource && (
        <div className="fixed inset-0 bg-black/75 backdrop-blur-sm z-50 flex items-center justify-center p-4 animate-fadeIn">
          <div className="bg-slate-900 border border-slate-800 rounded-2xl max-w-md w-full p-6 shadow-2xl space-y-5 animate-scaleUp">
            <div className="flex items-start gap-4">
              <div className="p-3 bg-red-500/10 text-red-400 border border-red-500/20 rounded-2xl flex-shrink-0">
                <AlertTriangle className="w-6 h-6" />
              </div>
              <div className="space-y-1">
                <h3 className="text-lg font-bold text-white">Supprimer la source ?</h3>
                <p className="text-slate-400 text-xs leading-relaxed">
                  Êtes-vous sûr de vouloir supprimer définitivement la source{' '}
                  <strong className="text-white font-semibold">« {deletingSource.name} »</strong> ?
                  Cette action supprimera également toute planification de veille et de scraping associée.
                </p>
              </div>
            </div>

            <div className="flex items-center justify-end gap-3 pt-3 border-t border-slate-800">
              <button
                type="button"
                disabled={deleteMutation.isPending}
                onClick={() => setDeletingSource(null)}
                className="px-4 py-2 text-xs font-semibold text-slate-400 hover:text-white rounded-xl transition cursor-pointer"
              >
                Annuler
              </button>
              <button
                type="button"
                disabled={deleteMutation.isPending}
                onClick={() => deletingSource._id && deleteMutation.mutate(deletingSource._id)}
                className="flex items-center gap-2 px-4 py-2 bg-red-600 hover:bg-red-500 disabled:opacity-50 text-white text-xs font-semibold rounded-xl shadow-lg shadow-red-600/20 transition cursor-pointer"
              >
                {deleteMutation.isPending ? (
                  <>
                    <RotateCw className="w-3.5 h-3.5 animate-spin" />
                    <span>Suppression en cours...</span>
                  </>
                ) : (
                  <>
                    <Trash2 className="w-3.5 h-3.5" />
                    <span>Confirmer la suppression</span>
                  </>
                )}
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
};
