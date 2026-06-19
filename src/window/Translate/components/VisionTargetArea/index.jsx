import {
    Button,
    ButtonGroup,
    Card,
    CardBody,
    CardFooter,
    CardHeader,
    Dropdown,
    DropdownItem,
    DropdownMenu,
    DropdownTrigger,
    Tooltip,
} from '@nextui-org/react';
import { writeText } from '@tauri-apps/api/clipboard';
import PulseLoader from 'react-spinners/PulseLoader';
import { MdContentCopy } from 'react-icons/md';
import { useTranslation } from 'react-i18next';
import { GiCycle } from 'react-icons/gi';
import ReactMarkdown from 'react-markdown';
import React, { useEffect, useState } from 'react';

import * as builtinServices from '../../../../services/vision';
import { getDisplayInstanceName, getServiceName, INSTANCE_NAME_CONFIG_KEY } from '../../../../utils/service_instance';

let visionRequestId = [];

export default function VisionTargetArea(props) {
    const { index, name, base64, visionServiceInstanceList, serviceInstanceConfigMap } = props;
    const [currentVisionServiceInstanceKey, setCurrentVisionServiceInstanceKey] = useState(name);
    const [isLoading, setIsLoading] = useState(false);
    const [result, setResult] = useState('');
    const [error, setError] = useState('');
    const { t } = useTranslation();

    function getInstanceName(instanceKey, serviceNameSupplier) {
        const instanceConfig = serviceInstanceConfigMap[instanceKey] ?? {};
        return getDisplayInstanceName(instanceConfig[INSTANCE_NAME_CONFIG_KEY], serviceNameSupplier);
    }

    const getBuiltinService = (instanceKey) => {
        return builtinServices[getServiceName(instanceKey)];
    };

    const analyze = async () => {
        const id = `${Date.now()}-${index}`;
        visionRequestId[index] = id;
        setResult('');
        setError('');
        setIsLoading(true);

        const service = getBuiltinService(currentVisionServiceInstanceKey);
        if (!service) {
            setError('Service not supported');
            setIsLoading(false);
            return;
        }

        const instanceConfig = serviceInstanceConfigMap[currentVisionServiceInstanceKey] ?? {};
        service
            .analyze(base64, {
                config: instanceConfig,
                setResult: (value) => {
                    if (visionRequestId[index] !== id) return;
                    setResult(value);
                },
            })
            .then(
                (value) => {
                    if (visionRequestId[index] !== id) return;
                    setResult(typeof value === 'string' ? value.trim() : value);
                    setIsLoading(false);
                },
                (e) => {
                    if (visionRequestId[index] !== id) return;
                    setError(e.toString());
                    setIsLoading(false);
                }
            );
    };

    useEffect(() => {
        if (base64 !== '' && currentVisionServiceInstanceKey) {
            analyze();
        }
    }, [base64, currentVisionServiceInstanceKey, serviceInstanceConfigMap]);

    const currentServiceName = getServiceName(currentVisionServiceInstanceKey);
    const currentService = getBuiltinService(currentVisionServiceInstanceKey);

    return (
        <Card
            shadow='none'
            className='rounded-[10px]'
        >
            <CardHeader className='flex justify-between py-1 px-0 bg-content2 rounded-t-[10px] h-[30px]'>
                <Dropdown>
                    <DropdownTrigger>
                        <Button
                            size='sm'
                            variant='solid'
                            className='bg-transparent'
                            startContent={
                                currentService ? (
                                    <img
                                        src={currentService.info.icon}
                                        className='h-[20px] w-[20px] my-auto'
                                        draggable={false}
                                    />
                                ) : null
                            }
                        >
                            <div className='my-auto'>
                                {currentService
                                    ? getInstanceName(currentVisionServiceInstanceKey, () =>
                                          t(`services.vision.${currentServiceName}.title`)
                                      )
                                    : currentVisionServiceInstanceKey}
                            </div>
                        </Button>
                    </DropdownTrigger>
                    <DropdownMenu
                        aria-label='vision service'
                        className='max-h-[40vh] overflow-y-auto'
                        onAction={(key) => {
                            setCurrentVisionServiceInstanceKey(key);
                        }}
                    >
                        {visionServiceInstanceList.map((instanceKey) => {
                            const serviceName = getServiceName(instanceKey);
                            const service = getBuiltinService(instanceKey);
                            const config = serviceInstanceConfigMap[instanceKey] ?? {};
                            const enable = config['enable'] ?? false;

                            return enable && service ? (
                                <DropdownItem
                                    key={instanceKey}
                                    startContent={
                                        <img
                                            src={service.info.icon}
                                            className='h-[16px] w-[16px] my-auto'
                                            draggable={false}
                                        />
                                    }
                                >
                                    {getInstanceName(instanceKey, () => t(`services.vision.${serviceName}.title`))}
                                </DropdownItem>
                            ) : null;
                        })}
                    </DropdownMenu>
                </Dropdown>
            </CardHeader>
            <CardBody className='bg-content1 px-[12px] py-[8px] min-h-[60px]'>
                {isLoading && result === '' && (
                    <div className='flex justify-center py-[10px]'>
                        <PulseLoader
                            size={8}
                            color='gray'
                        />
                    </div>
                )}
                {result !== '' && (
                    <div className='select-text whitespace-pre-wrap'>
                        <ReactMarkdown>{result}</ReactMarkdown>
                    </div>
                )}
                {error !== '' &&
                    error.split('\n').map((line) => {
                        return (
                            <p
                                key={line}
                                className='text-red-500'
                            >
                                {line}
                            </p>
                        );
                    })}
            </CardBody>
            <CardFooter className='bg-content1 rounded-none rounded-b-[10px] flex px-[12px] p-[5px]'>
                <ButtonGroup>
                    <Tooltip content={t('translate.copy')}>
                        <Button
                            isIconOnly
                            variant='light'
                            size='sm'
                            isDisabled={typeof result !== 'string' || result === ''}
                            onPress={() => {
                                writeText(result);
                            }}
                        >
                            <MdContentCopy className='text-[16px]' />
                        </Button>
                    </Tooltip>
                    <Tooltip content={t('translate.retry')}>
                        <Button
                            isIconOnly
                            variant='light'
                            size='sm'
                            onPress={() => {
                                analyze();
                            }}
                        >
                            <GiCycle className='text-[16px]' />
                        </Button>
                    </Tooltip>
                </ButtonGroup>
            </CardFooter>
        </Card>
    );
}
