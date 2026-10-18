import React, { useState } from 'react';
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { apiClient } from '../api/client';
import { Platform, SocialAccount, SourceCategory } from '../types';
import {
  Share2,
  Plus,
  ShieldCheck,
  AlertTriangle,
  RotateCw,
  Trash2,
  Lock,
  Twitter,
  Facebook,
  CheckCircle2,
  XCircle,
  Key,
} from 'lucide-react';

export const SocialAccountsView: React.FC = () => {
  const queryClient = useQueryClient();
  const [isAddModalOpen, setIsAddModalOpen] = useState(false);
  const [verifyFeedback, setVerifyFeedback] = useState<string | null>(null);

  // Formulaire d'ajout
  const [label, setLabel] = useState('');
  const [platform, setPlatform] = useState<Platform>('TWITTER');
  const [categories, setCategories] = useState<SourceCategory[]>(['NEWS', 'JOB_OFFER']);

  // Champs Twitter
  const [twitterApiKey, setTwitterApiKey] = useState('test_api_key');
  const [twitterApiSecret, setTwitterApiSecret] = useState('test_api_secret');
  const [twitterAccessToken, setTwitterAccessToken] = useState('test_access_token');
  const [twitterAccessTokenSecret, setTwitterAccessTokenSecret] = useState('test_token_secret');

  // Champs Facebook
  const [fbPageId, setFbPageId] = useState('test_page_123');
  const [fbAccessToken, setFbAccessToken] = useState('test_fb_token');
  const [fbAppId, setFbAppId] = useState('test_app_id');
  const [fbAppSecret, setFbAppSecret] = useState('test_app_secret');

  const { data: accounts = [], isLoading } = useQuery({
    queryKey: ['social-accounts'],
    queryFn: apiClient.getSocialAccounts,
  });

  // Action : Vérifier un compte
  const verifyMutation = useMutation({
    mutationFn: (id: string) => apiClient.verifySocialAccount(id),
    onSuccess: (data) => {
      setVerifyFeedback(
        data.is_valid
          ? '✓ Identifiants validés avec succès auprès du réseau !'
          : `✕ Échec de vérification : ${data.last_error}`
      );
      queryClient.invalidateQueries({ queryKey: ['social-accounts'] });
      queryClient.invalidateQueries({ queryKey: ['dashboard-stats'] });
      setTimeout(() => setVerifyFeedback(null), 6000);
    },
    onError: (err: any) => {
      setVerifyFeedback(`Erreur : ${err.response?.data?.error || err.message}`);
    },
  });

  // Action : Supprimer un compte
  const deleteMutation = useMutation({
    mutationFn: (id: string) => apiClient.deleteSocialAccount(id),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['social-accounts'] });
      queryClient.invalidateQueries({ queryKey: ['dashboard-stats'] });
    },
  });

  // Action : Créer un compte
  const createMutation = useMutation({
    mutationFn: (payload: any) => apiClient.createSocialAccount(payload),
    onSuccess: () => {
      setIsAddModalOpen(false);
      resetForm();
      queryClient.invalidateQueries({ queryKey: ['social-accounts'] });
      queryClient.invalidateQueries({ queryKey: ['dashboard-stats'] });
    },
  });

  const resetForm = () => {
    setLabel('');
    setPlatform('TWITTER');
    setCategories(['NEWS', 'JOB_OFFER']);
    setTwitterApiKey('test_api_key');
    setTwitterApiSecret('test_api_secret');
    setTwitterAccessToken('test_access_token');
    setTwitterAccessTokenSecret('test_token_secret');
    setFbPageId('test_page_123');
    setFbAccessToken('test_fb_token');
    setFbAppId('test_app_id');
    setFbAppSecret('test_app_secret');
  };

  const handleCreateSubmit = (e: React.FormEvent) => {
    e.preventDefault();

    let credentialsPayload: any = {};
    if (platform === 'TWITTER') {
      credentialsPayload = {
        auth_type: 'TwitterV2',
        api_key: twitterApiKey,
        api_secret: twitterApiSecret,
        access_token: twitterAccessToken,
        access_token_secret: twitterAccessTokenSecret,
      };
    } else if (platform === 'FACEBOOK') {
      credentialsPayload = {
        auth_type: 'FacebookGraph',
        page_id: fbPageId,
        page_access_token: fbAccessToken,
        app_id: fbAppId,
        app_secret: fbAppSecret,
      };
    }

    createMutation.mutate({
      label,
      platform,
      is_active: true,
      auto_publish_categories: categories,
      credentials: credentialsPayload,
    });
  };

  return (
    <div className="space-y-6 animate-fadeIn">
      {/* En-tête */}
      <div className="flex flex-col md:flex-row md:items-center justify-between gap-4">
        <div>
          <h1 className="text-3xl font-bold tracking-tight text-white flex items-center gap-3">
            <span>Comptes Sociaux & Passerelles</span>
            <span className="text-xs px-2.5 py-1 rounded-full bg-purple-500/10 text-purple-400 border border-purple-500/20 font-medium flex items-center gap-1">
              <Lock className="w-3 h-3" /> AES-256-GCM Chiffré
            </span>
          </h1>
          <p className="text-slate-400 text-sm mt-1">
            Gestion sécurisée des tokens API (X / Twitter, Facebook Graph) et statut de santé
          </p>
        </div>

        <button
          onClick={() => setIsAddModalOpen(true)}
          className="flex items-center gap-2 px-4 py-2.5 bg-indigo-600 hover:bg-indigo-500 text-white text-xs font-semibold rounded-xl transition shadow-lg shadow-indigo-600/20 self-start md:self-auto"
        >
          <Plus className="w-4 h-4" />
          <span>Connecter un Réseau</span>
        </button>
      </div>

      {/* Feedback vérification */}
      {verifyFeedback && (
        <div className="bg-slate-900 border border-purple-500/40 text-purple-200 p-4 rounded-xl text-xs flex items-center justify-between shadow-lg animate-fadeIn">
          <span>{verifyFeedback}</span>
          <button onClick={() => setVerifyFeedback(null)} className="text-slate-400 hover:text-white">
            ✕
          </button>
        </div>
      )}

      {/* Liste des comptes */}
      {isLoading ? (
        <div className="flex h-64 items-center justify-center">
          <div className="h-8 w-8 animate-spin rounded-full border-4 border-indigo-500 border-t-transparent" />
        </div>
      ) : accounts.length === 0 ? (
        <div className="bg-slate-900/60 border border-slate-800 rounded-2xl p-12 text-center">
          <Share2 className="w-12 h-12 text-slate-600 mx-auto mb-3" />
          <h3 className="text-slate-300 font-semibold">Aucun compte social connecté</h3>
          <p className="text-slate-500 text-sm mt-1 mb-4">
            Connectez votre compte Twitter ou page Facebook pour automatiser les diffusions.
          </p>
          <button
            onClick={() => setIsAddModalOpen(true)}
            className="px-4 py-2 bg-indigo-600 hover:bg-indigo-500 text-white text-xs font-semibold rounded-xl"
          >
            Connecter un compte
          </button>
        </div>
      ) : (
        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-5">
          {accounts.map((acc) => {
            const isTwitter = acc.platform === 'TWITTER';
            const isHealthy = acc.health_status.is_valid;

            return (
              <div
                key={acc.id}
                className="bg-slate-900 border border-slate-800 hover:border-slate-700 rounded-2xl p-5 shadow-lg space-y-4 transition"
              >
                {/* Entête compte */}
                <div className="flex items-center justify-between">
                  <div className="flex items-center gap-2.5">
                    <div
                      className={`p-2 rounded-xl ${
                        isTwitter
                          ? 'bg-sky-500/10 text-sky-400'
                          : 'bg-blue-600/10 text-blue-400'
                      }`}
                    >
                      {isTwitter ? <Twitter className="w-5 h-5" /> : <Facebook className="w-5 h-5" />}
                    </div>
                    <div>
                      <h3 className="font-bold text-white text-sm">{acc.label}</h3>
                      <span className="text-[11px] text-slate-400">{acc.platform} API</span>
                    </div>
                  </div>

                  <span
                    className={`flex items-center gap-1 px-2.5 py-0.5 rounded-full text-xs font-semibold ${
                      isHealthy
                        ? 'bg-emerald-500/10 text-emerald-400 border border-emerald-500/20'
                        : 'bg-red-500/10 text-red-400 border border-red-500/20'
                    }`}
                  >
                    {isHealthy ? <CheckCircle2 className="w-3 h-3" /> : <AlertTriangle className="w-3 h-3" />}
                    <span>{isHealthy ? 'Opérationnel' : 'Erreur'}</span>
                  </span>
                </div>

                {/* Clés masquées (Sécurité) */}
                <div className="bg-slate-950 p-3 rounded-xl border border-slate-800 text-xs space-y-1.5">
                  <div className="flex items-center justify-between text-slate-400">
                    <span className="flex items-center gap-1 text-[11px]">
                      <Key className="w-3 h-3 text-slate-500" /> Identifiants masqués
                    </span>
                    <span className="text-[10px] text-indigo-400 font-mono">Chiffré au repos</span>
                  </div>
                  <div className="font-mono text-[11px] text-slate-300 space-y-0.5">
                    {Object.entries(acc.credentials)
                      .filter(([k]) => k !== 'auth_type')
                      .map(([key, val]) => (
                        <div key={key} className="flex justify-between">
                          <span className="text-slate-500">{key}:</span>
                          <span className="text-slate-300 font-medium">{String(val)}</span>
                        </div>
                      ))}
                  </div>
                </div>

                {/* Catégories auto-publiées */}
                <div className="text-xs">
                  <span className="text-slate-400 block mb-1">Catégories autorisées :</span>
                  <div className="flex flex-wrap gap-1.5">
                    {acc.auto_publish_categories.map((cat) => (
                      <span
                        key={cat}
                        className="px-2 py-0.5 bg-slate-800 text-indigo-300 rounded font-medium text-[10px]"
                      >
                        {cat === 'NEWS' ? '📰 Actualités' : '💼 Emploi'}
                      </span>
                    ))}
                  </div>
                </div>

                {/* État de santé détaillé */}
                {acc.health_status.last_error && (
                  <div className="text-[11px] text-red-400 bg-red-950/30 p-2 rounded-lg border border-red-900/40">
                    Dernière erreur : {acc.health_status.last_error}
                  </div>
                )}

                {/* Actions */}
                <div className="pt-2 flex items-center justify-between gap-2 border-t border-slate-800">
                  <button
                    onClick={() => verifyMutation.mutate(acc.id)}
                    disabled={verifyMutation.isPending}
                    className="flex-1 flex items-center justify-center gap-1.5 py-2 px-3 bg-slate-800 hover:bg-slate-700 text-slate-200 text-xs font-semibold rounded-xl border border-slate-700 transition"
                  >
                    <RotateCw className="w-3.5 h-3.5" />
                    <span>Tester la connexion</span>
                  </button>

                  <button
                    onClick={() => deleteMutation.mutate(acc.id)}
                    className="p-2 text-slate-400 hover:text-red-400 hover:bg-red-950/30 rounded-xl transition"
                    title="Supprimer le compte"
                  >
                    <Trash2 className="w-4 h-4" />
                  </button>
                </div>
              </div>
            );
          })}
        </div>
      )}

      {/* Modal Ajout Compte */}
      {isAddModalOpen && (
        <div className="fixed inset-0 bg-black/70 backdrop-blur-sm z-50 flex items-center justify-center p-4">
          <div className="bg-slate-900 border border-slate-800 rounded-2xl max-w-lg w-full p-6 shadow-2xl space-y-5 animate-scaleUp">
            <div className="flex items-center justify-between pb-3 border-b border-slate-800">
              <h3 className="text-lg font-bold text-white flex items-center gap-2">
                <ShieldCheck className="w-5 h-5 text-indigo-400" />
                <span>Connecter un Réseau Social</span>
              </h3>
              <button
                onClick={() => setIsAddModalOpen(false)}
                className="text-slate-400 hover:text-white"
              >
                ✕
              </button>
            </div>

            <form onSubmit={handleCreateSubmit} className="space-y-4 text-xs">
              <div>
                <label className="block font-semibold text-slate-400 uppercase mb-1">
                  Nom du compte / Libellé
                </label>
                <input
                  type="text"
                  required
                  value={label}
                  onChange={(e) => setLabel(e.target.value)}
                  placeholder="Ex: Mon Compte Twitter Principal"
                  className="w-full bg-slate-950 border border-slate-800 rounded-xl p-2.5 text-white focus:outline-none focus:border-indigo-500"
                />
              </div>

              <div>
                <label className="block font-semibold text-slate-400 uppercase mb-1">
                  Plateforme
                </label>
                <select
                  value={platform}
                  onChange={(e) => setPlatform(e.target.value as Platform)}
                  className="w-full bg-slate-950 border border-slate-800 rounded-xl p-2.5 text-white focus:outline-none focus:border-indigo-500"
                >
                  <option value="TWITTER">Twitter / X (API v2)</option>
                  <option value="FACEBOOK">Facebook (Graph API)</option>
                </select>
              </div>

              {/* Notice simulation */}
              <div className="p-3 bg-indigo-950/30 border border-indigo-500/20 rounded-xl text-[11px] text-indigo-300">
                💡 <span className="font-semibold">Astuce test :</span> Des clés préfixées par <code className="bg-indigo-900/50 px-1 rounded">test_</code> ou <code className="bg-indigo-900/50 px-1 rounded">mock_</code> simulent une publication réussie sans appel réel.
              </div>

              {/* Formulaire spécifique Twitter */}
              {platform === 'TWITTER' && (
                <div className="space-y-3 p-3 bg-slate-950 rounded-xl border border-slate-800">
                  <span className="font-bold text-sky-400 uppercase text-[11px]">
                    Identifiants Twitter v2
                  </span>
                  <div>
                    <label className="block text-slate-400 mb-0.5">API Key</label>
                    <input
                      type="text"
                      required
                      value={twitterApiKey}
                      onChange={(e) => setTwitterApiKey(e.target.value)}
                      className="w-full bg-slate-900 border border-slate-800 rounded-lg p-2 text-white"
                    />
                  </div>
                  <div>
                    <label className="block text-slate-400 mb-0.5">API Secret</label>
                    <input
                      type="password"
                      required
                      value={twitterApiSecret}
                      onChange={(e) => setTwitterApiSecret(e.target.value)}
                      className="w-full bg-slate-900 border border-slate-800 rounded-lg p-2 text-white"
                    />
                  </div>
                  <div>
                    <label className="block text-slate-400 mb-0.5">Access Token (Bearer / User)</label>
                    <input
                      type="text"
                      required
                      value={twitterAccessToken}
                      onChange={(e) => setTwitterAccessToken(e.target.value)}
                      className="w-full bg-slate-900 border border-slate-800 rounded-lg p-2 text-white"
                    />
                  </div>
                  <div>
                    <label className="block text-slate-400 mb-0.5">Access Token Secret</label>
                    <input
                      type="password"
                      required
                      value={twitterAccessTokenSecret}
                      onChange={(e) => setTwitterAccessTokenSecret(e.target.value)}
                      className="w-full bg-slate-900 border border-slate-800 rounded-lg p-2 text-white"
                    />
                  </div>
                </div>
              )}

              {/* Formulaire spécifique Facebook */}
              {platform === 'FACEBOOK' && (
                <div className="space-y-3 p-3 bg-slate-950 rounded-xl border border-slate-800">
                  <span className="font-bold text-blue-400 uppercase text-[11px]">
                    Identifiants Facebook Graph API
                  </span>
                  <div>
                    <label className="block text-slate-400 mb-0.5">Page ID</label>
                    <input
                      type="text"
                      required
                      value={fbPageId}
                      onChange={(e) => setFbPageId(e.target.value)}
                      className="w-full bg-slate-900 border border-slate-800 rounded-lg p-2 text-white"
                    />
                  </div>
                  <div>
                    <label className="block text-slate-400 mb-0.5">Page Access Token</label>
                    <input
                      type="password"
                      required
                      value={fbAccessToken}
                      onChange={(e) => setFbAccessToken(e.target.value)}
                      className="w-full bg-slate-900 border border-slate-800 rounded-lg p-2 text-white"
                    />
                  </div>
                  <div className="grid grid-cols-2 gap-2">
                    <div>
                      <label className="block text-slate-400 mb-0.5">App ID</label>
                      <input
                        type="text"
                        required
                        value={fbAppId}
                        onChange={(e) => setFbAppId(e.target.value)}
                        className="w-full bg-slate-900 border border-slate-800 rounded-lg p-2 text-white"
                      />
                    </div>
                    <div>
                      <label className="block text-slate-400 mb-0.5">App Secret</label>
                      <input
                        type="password"
                        required
                        value={fbAppSecret}
                        onChange={(e) => setFbAppSecret(e.target.value)}
                        className="w-full bg-slate-900 border border-slate-800 rounded-lg p-2 text-white"
                      />
                    </div>
                  </div>
                </div>
              )}

              <div>
                <label className="block font-semibold text-slate-400 uppercase mb-1">
                  Catégories éligibles à la diffusion automatique
                </label>
                <div className="flex gap-4">
                  <label className="flex items-center gap-2 cursor-pointer">
                    <input
                      type="checkbox"
                      checked={categories.includes('NEWS')}
                      onChange={(e) => {
                        if (e.target.checked) setCategories([...categories, 'NEWS']);
                        else setCategories(categories.filter((c) => c !== 'NEWS'));
                      }}
                      className="rounded accent-indigo-500"
                    />
                    <span>Actualités (NEWS)</span>
                  </label>
                  <label className="flex items-center gap-2 cursor-pointer">
                    <input
                      type="checkbox"
                      checked={categories.includes('JOB_OFFER')}
                      onChange={(e) => {
                        if (e.target.checked) setCategories([...categories, 'JOB_OFFER']);
                        else setCategories(categories.filter((c) => c !== 'JOB_OFFER'));
                      }}
                      className="rounded accent-indigo-500"
                    />
                    <span>Offres d'Emploi (JOB_OFFER)</span>
                  </label>
                </div>
              </div>

              <div className="flex items-center justify-end gap-3 pt-3 border-t border-slate-800">
                <button
                  type="button"
                  onClick={() => setIsAddModalOpen(false)}
                  className="px-4 py-2 text-slate-400 hover:text-white"
                >
                  Annuler
                </button>
                <button
                  type="submit"
                  disabled={createMutation.isPending}
                  className="px-5 py-2.5 bg-indigo-600 hover:bg-indigo-500 text-white font-semibold rounded-xl shadow-lg transition"
                >
                  {createMutation.isPending ? 'Chiffrement & Sauvegarde...' : 'Enregistrer le compte'}
                </button>
              </div>
            </form>
          </div>
        </div>
      )}
    </div>
  );
};
