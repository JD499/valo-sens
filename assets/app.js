const API_ENDPOINTS = {
    PLAYERS: '/api/players'
};

const DEBOUNCE_DELAY = 300;
const state = {
    loading: false, cachedPlayers: [], selectedPro: null, infiniteScroll: {
        currentPage: 1, itemsPerPage: 30, hasMore: true, isLoading: false
    }, currentPlayers: [],

    convertList: {
        currentPage: 1, itemsPerPage: 15, hasMore: true, isLoading: false
    }
};

function createPlaceholder(count = 5) {
    const placeholder = document.createElement('div');
    placeholder.className = 'content-placeholder';


    for (let i = 0; i < count; i++) {
        const cardPlaceholder = document.createElement('div');
        cardPlaceholder.className = 'pro-card-placeholder';
        placeholder.appendChild(cardPlaceholder);
    }

    return placeholder;
}

const elements = {
    loading: document.getElementById('loading'),
    proList: document.getElementById('proList'),
    similarProList: document.getElementById('similarProList'),
    conversionResult: document.getElementById('conversionResult'),
    convertedValue: document.getElementById('convertedValue'),
    advancedOptions: document.getElementById('advancedOptions'),
    searchInput: document.getElementById('searchInput'),
    proSearch: document.getElementById('proSearch'),
    similarForm: document.getElementById('similarForm'),
    convertForm: document.getElementById('convertForm'),
    advancedToggle: document.getElementById('advancedToggle'),
    manualConvertBtn: document.getElementById('manualConvertBtn')
};


const createElement = (type, className, attributes = {}) => {
    const element = document.createElement(type);
    if (className) element.className = className;
    Object.entries(attributes).forEach(([key, value]) => element.setAttribute(key, value));
    return element;
};


const memoizedRenderProCard = (() => {
    const cache = new Map();

    return (pro, enableClick) => {
        const cacheKey = `${pro.name}-${pro.team}-${pro.sens}-${pro.dpi}-${enableClick}`;

        if (!cache.has(cacheKey)) {
            const card = renderProCard(pro, enableClick);
            cache.set(cacheKey, card);


            if (cache.size > 1000) {
                const firstKey = cache.keys().next().value;
                cache.delete(firstKey);
            }
        }

        return cache.get(cacheKey);
    };
})();

function validateElements() {
    const requiredElements = ['loading', 'proList', 'similarProList', 'conversionResult', 'convertedValue', 'advancedOptions', 'searchInput', 'proSearch', 'similarForm', 'convertForm', 'advancedToggle', 'manualConvertBtn'];

    const missingElements = requiredElements.filter(id => !elements[id]);
    if (missingElements.length > 0) {
        console.error('Missing required DOM elements:', missingElements);
        return false;
    }
    return true;
}

const tabs = {
    search: document.getElementById('searchTab'),
    similar: document.getElementById('similarTab'),
    convert: document.getElementById('convertTab')
};

const contents = {
    search: document.getElementById('searchContent'),
    similar: document.getElementById('similarContent'),
    convert: document.getElementById('convertContent')
};

function debounce(func, wait) {
    let timeout;
    return function executedFunction(...args) {
        clearTimeout(timeout);
        timeout = setTimeout(() => func.apply(this, args), wait);
    };
}

async function fetchJson(url, errorMessage = 'Error fetching data') {
    try {
        const response = await fetch(url);
        const data = await response.json();
        if (!data.success) throw new Error(data.error || 'API request failed');
        return data;
    } catch (error) {
        console.error(`${errorMessage}:`, error);
        throw error;
    }
}

function getInputValue(id) {
    return parseFloat(document.getElementById(id)?.value);
}

function scrollToElement(element, options = {behavior: 'smooth'}) {
    element?.scrollIntoView(options);
}


function setLoading(loading) {
    state.loading = loading;

    if (loading) {
        elements.proList.innerHTML = '';
        elements.proList.appendChild(createPlaceholder());
    }
}

function renderError(containerId, message) {
    const container = document.getElementById(containerId);
    if (container) {
        container.innerHTML = `<div class="error-message">${message}</div>`;
    }
}

function renderProCard(pro, enableClick = false) {
    return `
        <div class="pro-card ${enableClick ? 'cursor-pointer' : ''}"
             ${enableClick ? `data-pro='${JSON.stringify(pro)}'` : ''}
             role="listitem"
             aria-label="Player profile for ${pro.name}"
             ${enableClick ? 'tabindex="0"' : ''}
             ${enableClick ? 'aria-pressed="false"' : ''}
             ${enableClick ? `aria-description="Click to convert to ${pro.name}'s sensitivity settings"` : ''}>
            <div>
                <h3 role="heading" aria-level="3">${pro.name}</h3>
                <p>${pro.team}</p>
            </div>
            <div>
                <p>Sens: ${pro.sens.toFixed(3)}</p>
                <p>DPI: ${pro.dpi} | eDPI: ${pro.edpi.toFixed(1)}</p>
            </div>
        </div>
    `;
}


function renderProList(players, targetId, enableClick = false, append = false) {
    const container = document.getElementById(targetId);
    if (!container) return;


    if (!append) {
        container.innerHTML = '';
        container.appendChild(createPlaceholder());
    }


    if (!players.length && !append) {
        container.innerHTML = '<div role="listitem">No players found</div>';
        return;
    }

    const fragment = document.createDocumentFragment();
    const stateObj = targetId === 'convertProList' ? state.convertList : state.infiniteScroll;
    const start = (stateObj.currentPage - 1) * stateObj.itemsPerPage;
    const end = start + stateObj.itemsPerPage;
    const paginatedPlayers = players.slice(start, end);


    const existingSentinel = container.querySelector('.scroll-sentinel');
    existingSentinel?.remove();


    const html = paginatedPlayers.map(pro => memoizedRenderProCard(pro, enableClick)).join('');

    if (append) {
        const temp = createElement('div');
        temp.innerHTML = html;
        while (temp.firstChild) {
            fragment.appendChild(temp.firstChild);
        }
        container.appendChild(fragment);
    } else {
        container.innerHTML = html;
    }


    if (enableClick) {
        const newCards = container.querySelectorAll('[data-pro]:not([data-handler])');
        if (newCards.length) {
            const handler = (e) => {
                const card = e.target.closest('[data-pro]');
                if (card) {
                    state.selectedPro = JSON.parse(card.dataset.pro);
                    convertToProSettings();
                }
            };


            container.addEventListener('click', handler);
            newCards.forEach(el => el.setAttribute('data-handler', 'true'));
        }
    }


    if (players.length > end) {
        const sentinel = createElement('div', 'scroll-sentinel');
        container.appendChild(sentinel);

        const observer = targetId === 'convertProList' ? (window.convertScrollObserver || (window.convertScrollObserver = createConvertScrollObserver())) : (window.scrollObserver || (window.scrollObserver = createScrollObserver()));

        observer.observe(sentinel);
    }

    stateObj.hasMore = players.length > end;
}


const createBaseScrollObserver = (callback, rootMargin = '100px') => new IntersectionObserver(entries => entries.forEach(entry => entry.isIntersecting && callback()), {
    root: null, rootMargin, threshold: 0
});

function createScrollObserver() {
    return createBaseScrollObserver(() => !state.infiniteScroll.isLoading && state.infiniteScroll.hasMore && loadMorePlayers());
}

function createConvertScrollObserver() {
    return createBaseScrollObserver(() => !state.convertList.isLoading && state.convertList.hasMore && loadMoreConvertPlayers(), '50px');
}


async function loadMorePlayers() {
    console.log('loadMorePlayers called:', {
        isLoading: state.infiniteScroll.isLoading,
        hasMore: state.infiniteScroll.hasMore,
        currentPage: state.infiniteScroll.currentPage,
        totalPlayers: state.currentPlayers.length
    });

    if (state.infiniteScroll.isLoading || !state.infiniteScroll.hasMore) {
        console.log('Cancelled loadMorePlayers - already loading or no more content');
        return;
    }

    state.infiniteScroll.isLoading = true;

    try {
        state.infiniteScroll.currentPage++;
        console.log('Incremented page to:', state.infiniteScroll.currentPage);


        renderProList(state.currentPlayers, 'proList', false, true);
    } finally {
        state.infiniteScroll.isLoading = false;
        console.log('Finished loading more players');
    }
}


function searchPlayers(query = '') {
    const searchTerm = query.toLowerCase().trim();

    console.log('searchPlayers called with:', searchTerm);


    state.infiniteScroll.currentPage = 1;
    state.infiniteScroll.hasMore = true;


    state.currentPlayers = state.cachedPlayers.filter(pro => pro.name.toLowerCase().includes(searchTerm) || pro.team.toLowerCase().includes(searchTerm));

    console.log('Search results:', {
        totalResults: state.currentPlayers.length,
        hasMore: state.infiniteScroll.hasMore,
        currentPage: state.infiniteScroll.currentPage
    });

    return state.currentPlayers;
}


function switchTab(tabName) {
    Object.values(tabs).forEach(tab => tab.classList.remove('tab-active'));
    Object.values(contents).forEach(content => content.classList.add('hidden'));

    tabs[tabName].classList.add('tab-active');
    contents[tabName].classList.remove('hidden');
}

function findSimilarPlayers(targetEdpi, limit = 10) {
    return state.cachedPlayers
        .map(pro => ({
            ...pro, difference: Math.abs(pro.edpi - targetEdpi)
        }))
        .sort((a, b) => a.difference - b.difference)
        .slice(0, limit)
        .map(({difference, ...pro}) => pro);
}

async function handleSimilarFormSubmit(e) {
    e.preventDefault();
    const sens = getInputValue('similarSens');
    const dpi = getInputValue('similarDpi');

    if (!sens || !dpi) {
        renderError('similarProList', 'Please enter both DPI and sensitivity values.');
        return;
    }

    try {
        const targetEdpi = sens * dpi;
        const similarPros = findSimilarPlayers(targetEdpi);
        renderProList(similarPros, 'similarProList');
        scrollToElement(elements.similarProList);
    } catch (error) {
        console.error('Error finding similar sensitivities:', error);
        renderError('similarProList', 'Error finding similar sensitivities. Please try again.');
    }
}

function convertSensitivity(currentDpi, currentSens, targetDpi) {
    return (currentDpi * currentSens) / targetDpi;
}


function displayConvertedSensitivity(value) {
    if (!elements.conversionResult || !elements.convertedValue) return;

    elements.conversionResult.classList.remove('hidden');

    requestAnimationFrame(() => {
        elements.conversionResult.classList.add('visible');
        elements.convertedValue.textContent = value.toFixed(3);
    });
}

function convertToProSettings() {
    if (!state.selectedPro) return;

    const yourDpi = getInputValue('yourDpi');
    if (!yourDpi) {
        renderError('conversionResult', 'Please enter your DPI');
        return;
    }

    try {
        const convertedSens = convertSensitivity(state.selectedPro.dpi, state.selectedPro.sens, yourDpi);
        displayConvertedSensitivity(convertedSens);
    } catch (error) {
        console.error('Error converting sensitivity:', error);
        renderError('conversionResult', 'Error converting sensitivity. Please check your input values.');
    }
}

function handleManualConvert() {
    const currentDpi = getInputValue('yourDpi');
    const currentSens = getInputValue('yourSens');
    const targetDpi = getInputValue('targetDpi');

    if (!currentDpi || !currentSens || !targetDpi) {
        renderError('conversionResult', 'Please fill in all DPI and sensitivity values');
        return;
    }

    try {
        const convertedSens = convertSensitivity(currentDpi, currentSens, targetDpi);
        displayConvertedSensitivity(convertedSens);
    } catch (error) {
        console.error('Error converting sensitivity:', error);
        renderError('conversionResult', 'Error converting sensitivity. Please check your input values.');
    }
}

function toggleAdvancedOptions() {
    if (!elements.advancedOptions || !elements.advancedToggle) return;

    const isHidden = elements.advancedOptions.classList.toggle('hidden');
    const buttonText = isHidden ? 'Show Manual DPI Conversion' : 'Hide Manual DPI Conversion';

    const buttonSpan = elements.advancedToggle.querySelector('span');
    buttonSpan.textContent = buttonText;
}

function initializeEventListeners() {
    if (!validateElements()) {
        console.error('Failed to initialize event listeners due to missing DOM elements');
        return;
    }

    Object.entries(tabs).forEach(([name, tab]) => {
        if (tab) {
            tab.addEventListener('click', () => switchTab(name));
        }
    });

    elements.searchInput.addEventListener('input', debounce(e => {
        const filtered = searchPlayers(e.target.value);
        state.infiniteScroll.currentPage = 1;
        renderProList(filtered, 'proList', false, false);


        const sentinel = elements.proList.querySelector('.scroll-sentinel');
        if (sentinel) window.scrollObserver.observe(sentinel);
    }, DEBOUNCE_DELAY));


    elements.proSearch.addEventListener('input', debounce(e => {
        const filtered = searchPlayers(e.target.value);

        state.convertList.currentPage = 1;
        state.convertList.hasMore = true;
        renderConvertProList(filtered, 'convertProList', false);
    }, DEBOUNCE_DELAY));

    elements.similarForm.addEventListener('submit', handleSimilarFormSubmit);
    elements.convertForm.addEventListener('submit', e => {
        e.preventDefault();
        convertToProSettings();
    });

    elements.advancedToggle.addEventListener('click', toggleAdvancedOptions);
    elements.manualConvertBtn.addEventListener('click', handleManualConvert);
}


async function initializeProLists() {
    try {
        setLoading(true);
        const data = await fetchJson(API_ENDPOINTS.PLAYERS, 'Error fetching initial players');
        if (data.data) {
            state.cachedPlayers = data.data;
            state.currentPlayers = data.data;


            renderProList(state.currentPlayers, 'proList', false, false);
            renderConvertProList(state.currentPlayers, 'convertProList', false);
        }
    } catch (error) {
        console.error('Error initializing pro lists:', error);
        renderError('proList', 'Error loading initial player data. Please try again.');
    } finally {
        setLoading(false);
    }
}


function renderConvertProList(players, targetId, append = false) {
    const container = document.getElementById(targetId);
    if (!container) return;

    if (!players.length && !append) {
        container.innerHTML = '<div role="listitem">No players found</div>';
        return;
    }


    const start = (state.convertList.currentPage - 1) * state.convertList.itemsPerPage;
    const end = start + state.convertList.itemsPerPage;
    const paginatedPlayers = players.slice(start, end);


    state.convertList.hasMore = players.length > end;


    const existingSentinel = container.querySelector('.scroll-sentinel');
    if (existingSentinel) {
        existingSentinel.remove();
    }


    const html = paginatedPlayers.map(pro => renderProCard(pro, true)).join('');


    if (append) {
        container.insertAdjacentHTML('beforeend', html);
    } else {
        container.innerHTML = html;
    }


    container.querySelectorAll('[data-pro]:not([data-handler])').forEach(el => {
        el.setAttribute('data-handler', 'true');
        el.addEventListener('click', () => {
            state.selectedPro = JSON.parse(el.dataset.pro);
            convertToProSettings();
        });
    });


    if (state.convertList.hasMore) {
        const sentinel = document.createElement('div');
        sentinel.className = 'scroll-sentinel';
        container.appendChild(sentinel);


        if (!window.convertScrollObserver) {
            window.convertScrollObserver = createConvertScrollObserver();
        }
        window.convertScrollObserver.observe(sentinel);
    }
}


async function loadMoreConvertPlayers() {
    if (state.convertList.isLoading || !state.convertList.hasMore) return;

    state.convertList.isLoading = true;

    try {
        state.convertList.currentPage++;
        renderConvertProList(state.currentPlayers, 'convertProList', true);
    } finally {
        state.convertList.isLoading = false;
    }
}


async function initialize() {
    try {
        if (!validateElements()) {
            throw new Error('Required DOM elements are missing');
        }


        window.scrollObserver = createScrollObserver();

        initializeEventListeners();
        await initializeProLists();
    } catch (error) {
        console.error('Initialization error:', error);
    }
}

initialize();